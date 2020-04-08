use clap::clap_app;
use serde::{Deserialize, Serialize};
mod protocol;

fn main() {
    let matches = clap_app!(remitscli =>
        (version: "0.1")
        (about: "Interact with remits")
        (@arg debug: -d ... "Sets the level of debugging information")
        (@subcommand log_add =>
            (about: "Add log")
            (@arg log_name: -n +required +takes_value "Log name to add")
        )
        (@subcommand log_list =>
            (about: "List logs")
        )
        (@subcommand log_del =>
            (about: "Delete log")
            (@arg log_name: -n +required +takes_value "Log name to delete")
        )
        (@subcommand log_show =>
            (about: "Show metadata of log")
            (@arg log_name: -n +required +takes_value "Log name to see metadata")
        )
        (@subcommand msg_add =>
            (about: "Add message to log")
            (@arg msg: -m +takes_value "Value of msg to add")
        )
        (@subcommand iterator_add =>
            (about: "Add iterator to log")
            (@arg log: -l +required +takes_value "Value of log to add iterator")
            (@arg iterator_name: -n +required +takes_value "choose iterator name")
            (@arg iterator_type: -t +required +takes_value "select iterator type")
        )
        (@subcommand iterator_list =>
            (about: "List all iterators")
        )
        (@subcommand iterator_next =>
            (about: "Get up to <count> messages from an Iterator")
            (@arg iterator_name: -n +required +takes_value "iterator name")
            (@arg message_id: -i +required +takes_value "message_id")
            (@arg count: -c +required +takes_value "count")
        )
    )
    .get_matches();

    let request = match matches.subcommand() {
        ("log_list", Some(_)) => protocol::new_log_list_req(),
        ("log_add", Some(args)) => protocol::new_log_add_req(args.value_of("log_name").unwrap()),
        ("log_show", Some(args)) => protocol::new_log_show_req(args.value_of("log_name").unwrap()),
        ("log_del", Some(args)) => protocol::new_log_del_req(args.value_of("log_name").unwrap()),
        ("msg_add", Some(args)) => {
            #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
            struct Msg {
                data: String,
            }
            let test_msg = Msg {
                data: args.value_of("msg").unwrap().into(),
            };
            let cbor = serde_cbor::to_vec(&test_msg).unwrap();

            protocol::new_msg_add_req("test", cbor)
        }
        ("iterator_add", Some(args)) => {
            let log = args.value_of("log").unwrap();
            let iterator_name = args.value_of("iterator_name").unwrap();
            let iterator_type = args.value_of("iterator_type").unwrap();
            protocol::new_iterator_add_req(log, iterator_name, iterator_type)
        }
        ("iterator_list", Some(_)) => protocol::new_iterator_list_req(),
        ("iterator_next", Some(args)) => {
            let iterator_name = args.value_of("iterator_name").unwrap();
            let message_id = args.value_of("message_id").unwrap().parse().unwrap();
            let count = args.value_of("count").unwrap().parse().unwrap();
            protocol::new_iterator_next_req(iterator_name, message_id, count)
        }
        _ => panic!("{}", "Type help, -h, or --help"),
    };
    let output = protocol::send_req(request);
    if output.0 == 0x03 {
        println!("ERROR OCCURED");
        println!("Response from remits {:?}", output.2);
        panic!("Bad request");
    }
    if output.0 == 0x01 {
        println!("Info response");
    }
    if output.0 == 0x02 {
        println!("Info response");
    }
    if output.2 == protocol::OK_RESP {
        println!("Ok response");
    }
    println!("Response from remits {:?}", output.2);
}
