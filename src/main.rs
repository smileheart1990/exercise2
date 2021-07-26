use crate::strategy::Back_test;
mod strategy;
use std::collections::HashMap;
fn main() {
    let mut test_strategy = strategy::Strategy_input{
        data_folder: String::from("./v3_kline_2021_06_23/"),
        //format "%Y%m%d%H%M";
        start_time: String::from("202106230000"),
        end_time: String::from("202106230900"),
    };
    test_strategy.back_test();
}
