use once_cell::sync::Lazy;
use std::collections::HashMap;

use serenity::model::{
    prelude::interaction::application_command::{
        ApplicationCommandInteraction, CommandDataOptionValue,
    },
    user::User,
};
use uuid::Uuid;

static mut ACTIVE_BOUNTIES: Lazy<HashMap<Uuid, Bounty>> = Lazy::new(|| {
    let map = HashMap::new();
    // Perform any initialization if needed
    map
});

struct Bounty {
    lister: User,
    hunter: User,
    bounty_number: u32,
    middlemen: Vec<User>,
}

impl Bounty {
    pub fn new(lister: User, hunter: User, bounty_number: u32) -> Bounty {
        Bounty {
            lister: lister.clone(),
            hunter: hunter.clone(),
            bounty_number,
            middlemen: Vec::new(),
        }
    }
}

pub fn handle_bounty(input: ApplicationCommandInteraction) {
    let id = Uuid::new_v4();

    let mut hunter: User = User::default();
    let mut number: u32 = 0;
    for args in input.data.options {
        match args.name.as_str() {
            "hunter" => {
                if let Some(arg) = args.resolved {
                    if let CommandDataOptionValue::User(user, _) = arg {
                        hunter = user.clone();
                    }
                }
            }
            "number" => {
                if let Some(arg) = args.resolved {
                    if let CommandDataOptionValue::Integer(val) = arg {
                        if val >= 0 && val <= u32::MAX as i64 {
                            number = val as u32;
                        } else {
                            eprintln!("Invalid bounty number");
                        }
                    }
                }
            }
            _ => eprintln!("Unknown argument"),
        }
    }

    let bounty = Bounty::new(input.user, hunter, number);

    ACTIVE_BOUNTIES.insert(id, bounty);

    // let new_bounty = Bounty::new(input.user);
}
