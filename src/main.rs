extern crate docopt;
extern crate rustc_serialize;
extern crate kafka;

use docopt::Docopt;
use kafka::client::KafkaClient;

// usage string
static USAGE: &'static str = "
kcat - cat for Kafka.

Usage:
    kcat [-b BROKER] -t TOPIC
    kcat (-h | --help)

Options:
    -h, --help                  Show this message.
    -b BROKER, --broker=BROKER  Kafka broker [default: localhost:9092].
    -t TOPIC, --topic=TOPIC     Kafka topic.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    flag_broker: String,
    flag_topic: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let mut client = KafkaClient::new(vec!(args.flag_broker));
    let meta_res = client.load_metadata_all();
    if meta_res.is_err() {
        match meta_res.err() {
            Some(err) => println!("Error: {}", err),
            None => println!("Non-specific error")
        }
        return;
    }

    let mut prev_offset:i64 = -1;
    loop {
        let res = client.fetch_messages(
            args.flag_topic.to_string(), // topic
            0,                           // partition
            prev_offset + 1              // offset
        );
        if res.is_ok() {
            for tmsg in res.unwrap() {
                println!("{}", String::from_utf8(tmsg.message).unwrap());
                prev_offset = tmsg.offset;
            }
        }
    }
}
