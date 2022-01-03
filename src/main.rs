mod credentials;

use ads1x1x::ic::{Ads1115, Resolution16Bit};
use ads1x1x::interface::I2cInterface;
use ads1x1x::mode;
use ads1x1x::{channel, Ads1x1x, DataRate16Bit, SlaveAddr};
use embedded_hal::adc::OneShot;
use linux_embedded_hal::I2cdev;
use nb::block;
use paho_mqtt as mqtt;
use std::{process, thread, time};

struct BatteryMonitor {
    adc: Ads1x1x<I2cInterface<I2cdev>, Ads1115, Resolution16Bit, mode::OneShot>,
}

struct BatteryStatus {
    volts: f32,
    charge: f32,
}

impl BatteryMonitor {
    pub fn new() -> Self {
        let dev = I2cdev::new("/dev/i2c-1").unwrap();
        let mut adc = Ads1x1x::new_ads1115(dev, SlaveAddr::default());
        adc.set_data_rate(DataRate16Bit::Sps8).unwrap();
        return BatteryMonitor { adc: adc };
    }

    pub fn read(&mut self) -> BatteryStatus {
        let cycles = 40;
        let mut sum = 0.0;

        for _ in 0..cycles {
            let measurement = block!(self.adc.read(&mut channel::SingleA0)).unwrap();
            let volts = 13.09 * ((measurement as f32) - 9.0) / 18769.0;
            sum += volts;
        }

        let volts = sum / (cycles as f32);
        let charge = self.volts_to_charge(volts);

        BatteryStatus {
            volts: volts,
            charge: charge,
        }
    }

    pub fn volts_to_charge(&self, volts: f32) -> f32 {
        let max_v = 13.0;
        let min_v = 11.0;
        let mut v = volts;
        v = v.min(max_v);
        v = v.max(min_v);

        100.0 * (v - min_v) / (max_v - min_v)
    }
}
fn main() {
    // pub fn destroy(&mut self) {
    //    let _dev = self.adc.destroy_ads1115(); // get I2C device back
    //}
    let mut bm = BatteryMonitor::new();

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
