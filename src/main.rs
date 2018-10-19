#[macro_use] extern crate clap;
extern crate futures;

extern crate rdkafka;

use clap::{Arg, App};
use futures::stream::Stream;
use rdkafka::message::{Message, Headers};
use rdkafka::client::ClientContext;
use rdkafka::consumer::{Consumer, ConsumerContext, CommitMode, DefaultConsumerContext, Rebalance};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::util::get_rdkafka_version;
use rdkafka::error::KafkaResult;
use std::io;
use std::io::prelude::*;

fn main() {
    let args = App::new("kcat")
        .version(crate_version!())
        .about("cat for Apache Kafka")
        .arg(Arg::with_name("broker")
            .short("b")
            .long("broker")
            .value_name("BROKER")
            .help("Kafka broker")
            .takes_value(true))
        .arg(Arg::with_name("topic")
            .short("t")
            .long("topic")
            .value_name("TOPIC")
            .help("Kafka topic")
            .takes_value(true)
            .required(true))
        .get_matches();
    let broker = args.value_of("broker").unwrap_or("localhost:9092");
    let topic = args.value_of("topic").unwrap();
    let topics = vec![topic];

    let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", "kcat")
        .set("bootstrap.servers", broker)
        .set("enable.partition.eof", "false")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "smallest")
        .create()
        .expect("Failed to create consumer");
    consumer.subscribe(&topics)
        .expect("Failed to subscript to topic");
    let message_stream = consumer.start();

    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout.lock());
    let _ = writeln!(writer, "fetching messages...").unwrap();
    writer.flush().unwrap();
    for message in message_stream.wait() {
        match message {
            Err(_) => writeln!(&mut io::stderr(), "Error while reading from stream.").unwrap(),
            Ok(Err(e)) => writeln!(&mut io::stderr(), "Kafka error: {}", e).unwrap(),
            Ok(Ok(m)) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        writeln!(&mut io::stderr(), "Error while deserializing message payload: {:?}", e);
                        ""
                    },
                };
                writeln!(writer, "{}", payload).unwrap();
                writer.flush().unwrap();
                //consumer.commit_message(&m, CommitMode::Async).unwrap();
            },
        };
    }

}
