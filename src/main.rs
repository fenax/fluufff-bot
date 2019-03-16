extern crate futures;
extern crate telegram_bot_fork;
extern crate tokio;

use std::env;
use std::collections::VecDeque;

use futures::{Stream, future::lazy};

use telegram_bot_fork::*;


struct AdminChat{
    admin: Box<User>,
    title: String,
    current_chat: Option<Box<User>>,
    waiting_line: VecDeque<User>,
}

impl AdminChat{
    pub fn new(title:&str,admin:&User,api:&Api)->AdminChat{
        api.spawn(admin.text(format!("you are now {}\nType /quit to stop being it.",title)));
        AdminChat {
            admin:Box::new(admin.clone()),
            title:title.to_string(),
            current_chat:None,
            waiting_line:VecDeque::new(),
        }
    }

    fn set_current_chat(&mut self, user:&User,api:&Api){
        self.current_chat = Some(Box::new(user.clone()));
        api.spawn(user.text(format!("you are now talking to {}.\nThe conversation can be closed with /close",self.title)));
        api.spawn(self.admin.text(format!("you are now talking to {}.\nThe conversation can be closed with /close", user.first_name)));
    }

    fn ask_chat(&mut self, user :&User,api:&Api){
        match &self.current_chat{
            None => {
                self.set_current_chat(&user,&api);
            },
            Some(a) => {
                if **a == *user {
                    api.spawn(user.text(format!("what are you doing, you are already talking to {}",self.title)));
                }else if self.waiting_line.contains(user){
                    api.spawn(user.text("you are already in the waiting list"));
                }else{
                    self.waiting_line.push_back(user.clone());
                    api.spawn(user.text("you are on waiting list"));
                    api.spawn(self.admin.text(format!("{} is now on waiting list",user.first_name)));
                }
            }
        }
    }
    fn end_current_chat(&mut self,api:&Api){
        if let Some(c) = &self.current_chat {
            api.spawn(c.text(format!("your conversation with {} was clotured",self.title)));
            api.spawn(self.admin.text(format!("your conversation with {} was clotured",c.first_name)));
            self.current_chat = None;
            match self.waiting_line.pop_front(){
                None => {
                    
                },
                Some(a) =>{
                    self.set_current_chat(&a,&api);
                },
            }
        }
    }
}


fn main() {
    tokio::runtime::current_thread::Runtime::new().unwrap().block_on(lazy(|| {

        
        
        let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
        let api = Api::new(token).unwrap();

        let mut conops :Option<AdminChat> = None;
        // Convert stream to the stream with errors in result
        let stream = api.stream().then(|mb_update| {
            let res: Result<Result<Update, Error>, ()> = Ok(mb_update);
            res
        });

        // Print update or error for each update.
        stream.for_each(move |update| {
            match update {
                Ok(update) => {
                    // If the received update contains a new message...
                    if let UpdateKind::Message(message) = update.kind {
                        if let MessageKind::Text { ref data, .. } = message.kind {
                            match data.as_ref(){
                                "/start" => {
                                    api.spawn(message.chat.text("Hello, welcome to Fluufff telegram bot.\n Please type /conops to talk to conops."));
                                }
                                "/conops" => {
                                    match conops.as_mut() {
                                        Some(a) => { a.ask_chat(&message.from,&api); },
                                        None => {
                                            api.spawn(message.chat.text("conops is not available"));
                                        },
                                    }
                                },
                                "/reg" => {
                                },
                                "/iam conops" =>{
                                    conops = Some(AdminChat::new( "conops",&message.from,&api));
                                },
                                "/close" =>{
                                    if let Some(a) = &mut conops {
                                        if message.from == *a.admin {
                                            a.end_current_chat(&api);
                                        }else{
                                            if let Some(b) = &a.current_chat {
                                                if **b == message.from{
                                                    a.end_current_chat(&api);
                                                }
                                            }
                                        }
                                    }
                                },
                                "/quit" =>{
                                    if let Some(a) = &conops {
                                        if message.from == *a.admin {
                                            conops = None;
                                        }
                                    }
                                },
                                _ =>{
                                    if let Some(a) = &conops {
                                        if message.from == *a.admin{
                                            if let Some(b) = &a.current_chat{
                                                api.spawn(b.text(data));
                                            }else{
                                                api.spawn(message.chat.text("You are not yet talking to anybody yet, you can type /conops to talk to conops."));
                                            }
                                        }
                                        match &a.current_chat{
                                        Some(b) =>{
                                            if message.from == **b{
                                               api.spawn(message.forward(a.admin.clone()));
                                            }
                                        },
                                        None => {}}
                                    }
                            
                                }}

                                
                            // Print received text message to stdout.
                            println!("<{}>: {}", &message.from.first_name, data);

                            // Answer message with "Hi".
            //                api.spawn(message.text_reply(format!(
            //                    "Hi, {}! You just wrote '{}'",
            //                    &message.from.first_name, data
            //                )));
                        }
                    }
                }
                Err(_) => {}
            }

            Ok(())
        })
    })).unwrap();

}
