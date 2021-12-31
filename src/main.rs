use ads1x1x::ic::{Ads1115, Resolution16Bit};
use ads1x1x::interface::I2cInterface;
use ads1x1x::mode;
use ads1x1x::{channel, Ads1x1x, DataRate16Bit, SlaveAddr};
use embedded_hal::adc::OneShot;
use linux_embedded_hal::I2cdev;
use nb::block;

struct BatteryMonitor {
    adc: Ads1x1x<I2cInterface<I2cdev>, Ads1115, Resolution16Bit, mode::OneShot>,
}

impl BatteryMonitor {
    pub fn new() -> Self {
        let dev = I2cdev::new("/dev/i2c-1").unwrap();
        let mut adc = Ads1x1x::new_ads1115(dev, SlaveAddr::default());
        adc.set_data_rate(DataRate16Bit::Sps8).unwrap();
        return BatteryMonitor { adc: adc };
    }

    pub fn read_volts(&mut self) -> f32 {
        let cycles = 40;
        let mut sum = 0.0;

        for _ in 0..cycles {
            let measurement = block!(self.adc.read(&mut channel::SingleA0)).unwrap();
            let volts = 13.09 * ((measurement as f32) - 9.0) / 18769.0;
            sum += volts;
        }

        sum / (cycles as f32)
    }

    // pub fn destroy(&mut self) {
    //    let _dev = self.adc.destroy_ads1115(); // get I2C device back
    //}
}

fn main() {
    let mut bm = BatteryMonitor::new();
    loop {
        let volts = bm.read_volts();
        println!("Measurement: batt_volts={}", volts);
    }
}
