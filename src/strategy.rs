use chrono::{DateTime, Utc, NaiveDateTime, NaiveDate, NaiveTime, Duration};
use lzma::LzmaReader;
use std::io::prelude::*;
use std::fs::File;
use std::{cell::RefCell, collections::HashMap};
use std::fmt;
enum K_line_frequency {
    MIN_1,
    MIN_15,
}
#[derive(Debug)]
pub struct Instrument_status{
    exchange:String,
    preCoin: String,
    postCoin:String,
    // o intial
    preCoin_balance : f64,
    // 1 initial
    postCoin_balance :f64,
    price: f64,
}
impl fmt::Display for Instrument_status{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exchange: {}, preCoin: {}, postCoin: {}, postCoin_balance: {}, preCoin_balance:{}", self.exchange, self.preCoin, self.postCoin, self.postCoin_balance, self.preCoin_balance)
    }
}
#[derive(Debug)]
pub struct Strategy_input {
    pub data_folder: String,
    pub start_time: String,
    pub end_time: String,
}

pub trait Back_test {
    fn back_test(&mut self);
}

impl Back_test for Strategy_input {
    fn back_test(&mut self) {
        let fmt = "%Y%m%d%H%M";
        let mut start_naive_datetime = NaiveDateTime::parse_from_str(&self.start_time[..], fmt).unwrap();
        let end_naive_datetime = NaiveDateTime::parse_from_str(&self.end_time[..], fmt).unwrap();
        let mut first_tick_or_not = true;
        let mut instrument_status_map: HashMap<String, RefCell<Instrument_status>> = HashMap::new();
        while start_naive_datetime <= end_naive_datetime {
            // need to know the file path, if the folder end with /, no need to add the folder
            let current_folder_and_file_name = if self.data_folder.ends_with('/'){
                self.data_folder.clone() + &start_naive_datetime.format("v3_kline_%Y_%m_%d_%H_%M.xz").to_string()
            }else {
                self.data_folder.clone() + &start_naive_datetime.format("/v3_kline_%Y_%m_%d_%H_%M.xz").to_string()
            };
            //println!("current time  file is {}", current_folder_and_file_name);
            //read the parse the file
            let f = File::open(current_folder_and_file_name).unwrap();
            let mut f = LzmaReader::new_decompressor(f).unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            let mut lines = s.split("\n");
            //read by line
            for s in lines {
                // println!("{}nth line", line_th);
                let values: Vec<&str> = s.split('\t').collect();
                // bad input data, will skip
                if(values.len()<10){
                    continue;
                }
                let exchange = values[2];
                let pre_coin = values[3];
                let post_coin = values[4];
                let end_price = values[9].parse::<f64>();
                let key = String::from(exchange)+ "_" + pre_coin +"_" + post_coin;
                //current tick data from the file
                let current_instrument_status = Instrument_status{
                    exchange:String:: from(exchange),
                    preCoin: String::from(pre_coin),
                    postCoin:String::from(post_coin),
                    // o intial
                    preCoin_balance : 0.0,
                    // 1 initial
                    postCoin_balance :1.0,
                    price: end_price.unwrap(),
                };
                if(first_tick_or_not){
                    instrument_status_map.insert(key, RefCell::new(current_instrument_status));
                }else{
                    let mut last_instrument_status =  instrument_status_map.get_mut(&key);

                    match last_instrument_status{
                        Some(last_instrument_status)=>{
                            let mut last_instrument_status_instance = last_instrument_status.borrow_mut();
                            if(last_instrument_status_instance.postCoin_balance != 0.0){
                                if(current_instrument_status.price/last_instrument_status_instance.price <1.01){
                                    last_instrument_status_instance.price = current_instrument_status.price;
                                    continue;
                                }else{
                                    last_instrument_status_instance.preCoin_balance = last_instrument_status_instance.postCoin_balance/current_instrument_status.price;
                                    last_instrument_status_instance.postCoin_balance =0.0;
                                    last_instrument_status_instance.price = current_instrument_status.price;
                                    continue;
                                }
                            }else if(last_instrument_status_instance.preCoin_balance != 0.0) {
                                last_instrument_status_instance.postCoin_balance = last_instrument_status_instance.preCoin_balance* current_instrument_status.price;
                                last_instrument_status_instance.preCoin_balance = 0.0;
                                last_instrument_status_instance.price = current_instrument_status.price;
                            }
                        },
                        Node => {}
                    }

                }
            }
            first_tick_or_not = false;
            start_naive_datetime = start_naive_datetime + Duration::minutes(1);
        }
        //bellow is to convert to the post coin base p&l
        // and then print out the result
        for (_, mut value) in &instrument_status_map {
            let mut current_value = value.borrow_mut();
            if(current_value.preCoin_balance !=0.0){
                current_value.postCoin_balance = current_value.postCoin_balance + current_value.preCoin_balance/ current_value.price;
            }
            println!("exchange:{}, pre_coin:{}, post_coin:{}, p&l:{} {}", current_value.exchange, current_value.preCoin, current_value.postCoin, current_value.postCoin_balance -1.0, current_value.postCoin)
        }

    }
}