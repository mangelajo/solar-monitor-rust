mod battery_monitor;
mod credentials;

use paho_mqtt as mqtt;
use std::{process, thread, time};

fn main() {
    let mut bm = battery_monitor::BatteryMonitor::new();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(credentials::SERVER)
        .client_id(credentials::CLIENT_ID)
        .max_buffered_messages(100)
        .finalize();

    let cli = mqtt::Client::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .user_name(credentials::USER)
        .password(credentials::PASSWORD)
        .finalize();

    cli.connect(conn_opts).unwrap();

    loop {
        let status = bm.read();

        println!(
            "Measurement: batt_volts={:.2}, charge={:.2}%",
            status.volts, status.charge
        );

        let msg = mqtt::MessageBuilder::new()
            .topic("doremor/caseta/batt/volts")
            .payload(format!("{:.2}", status.volts))
            .qos(1)
            .retained(true)
            .finalize();

        if let Err(e) = cli.publish(msg) {
            println!("Error sending message: {:?}", e);
        }

        let msg_charge = mqtt::MessageBuilder::new()
            .topic("doremor/caseta/batt/charge")
            .payload(format!("{:.2}", status.charge))
            .qos(1)
            .retained(true)
            .finalize();

        if let Err(e) = cli.publish(msg_charge) {
            println!("Error sending message: {:?}", e);
        }
        thread::sleep(time::Duration::from_secs(50));
    }
}
