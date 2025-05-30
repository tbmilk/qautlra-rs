use std::net::SocketAddrV4;
use std::sync::Arc;
use std::thread::{sleep, spawn};
use std::time::Duration;

use dotenv::dotenv;
use env_logger::init;
use failure::Fallible;
use log::{error, warn};
use structopt::StructOpt;
use xtp::{
    OrderBookStruct, QuoteApi, QuoteSpi, XTPExchangeType, XTPLogLevel, XTPMarketDataStruct,
    XTPProtocolType, XTPRspInfoStruct, XTPSpecificTickerStruct, XTPTickByTickStruct,
};

type XTPST = XTPSpecificTickerStruct;
type XTPRI = XTPRspInfoStruct;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of xtp-rs usage.")]
struct Args {
    #[structopt(short, long, default_value = "1")]
    id: i8,
    #[structopt(short, long, env = "XTP_SERVER_ADDR")]
    server_addr: SocketAddrV4,
    #[structopt(short, long, env = "XTP_USERNAME")]
    username: String,
    #[structopt(short, long, env = "XTP_PASSWORD")]
    password: String,
    #[structopt(long, default_value = "/tmp")]
    path: String,
}

fn main() -> Fallible<()> {
    let _ = dotenv();
    init();

    let args = Args::from_args();

    let mut api = QuoteApi::new(1, &args.path, XTPLogLevel::Trace);

    println!("XTP Version: {:?}", api.get_api_version());
    println!("Trading Day: {:?}", api.get_trading_day());

    api.register_spi(MySpi);

    api.set_heart_beat_interval(10);
    api.set_udp_buffer_size(1024);

    api.login(
        args.server_addr,
        &args.username,
        &args.password,
        XTPProtocolType::TCP,
    )?;
    let codes_sh = [
        "600036", "600000", "600001", "600018", "600009", "510050", "688300", "688310", "688100",
        "603777", "600570", "600030", "600019", "601519", "603050", "600001", "600002", "600004",
        "600010", "600871", "600861", "600855", "600816", "600817", "600818", "600819", "600820",
        "600821", "600822", "600823", "600824", "600825", "600826", "600827", "600710", "600711",
        "600712", "600713", "600714", "600715", "600716", "600717", "600718", "600719", "600720",
        "600721",
    ];
    let codes_sz = [
        "000001", "000002", "000006", "000007", "000008", "300339", "300380", "000977", "300001",
        "000016", "300180",
    ];

    let a = Arc::new(api);

    let mut v = vec![];
    for code in codes_sh.iter() {
        let a = a.clone();
        let code = code.to_string();
        let h = spawn(
            move || match a.subscribe_tick_by_tick(&[&code], XTPExchangeType::SH) {
                Ok(_) => println!("Subscribe {} success", code),
                Err(e) => error!("Subscribe {} Failed: {}", code, e),
            },
        );
        v.push(h);
    }
    for code in codes_sz.iter() {
        let a = a.clone();
        let code = code.to_string();
        let h = spawn(
            move || match a.subscribe_tick_by_tick(&[&code], XTPExchangeType::SZ) {
                Ok(_) => println!("Subscribe {} success", code),
                Err(e) => error!("Subscribe {} Failed: {}", code, e),
            },
        );
        v.push(h);
    }

    v.into_iter()
        .map(|h| h.join())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    sleep(Duration::from_secs(10));
    a.logout()?;
    Ok(())
}

struct MySpi;

impl QuoteSpi for MySpi {
    fn on_error(&self, error_info: XTPRI) {
        error!("{:?}", error_info);
    }

    fn on_disconnected(&self, reason: i32) {
        warn!("Disconnected, reason: {}", reason)
    }

    fn on_sub_market_data(&self, ticker: XTPST, error_info: XTPRI, is_last: bool) {
        println!(
            "Sub Market Data: {:?}: {:?} {}",
            ticker, error_info, is_last
        );
    }

    fn on_sub_order_book(&self, ticker: XTPST, error_info: XTPRI, is_last: bool) {
        println!("Sub Orderbook: {:?}: {:?} {}", ticker, error_info, is_last);
    }

    fn on_sub_tick_by_tick(&self, ticker: XTPST, error_info: XTPRI, is_last: bool) {
        println!(
            "Sub Tick By Tick: {:?}: {:?} {}",
            ticker, error_info, is_last
        );
    }

    fn on_depth_market_data(
        &self,
        market_data: XTPMarketDataStruct,
        bid1_qty: &[i64],
        max_bid1_count: i32,
        ask1_qty: &[i64],
        max_ask1_count: i32,
    ) {
        println!(
            "Market Depth: {:?}, {:?}, {}, {:?}, {}",
            market_data, bid1_qty, max_bid1_count, ask1_qty, max_ask1_count
        );
    }
    fn on_tick_by_tick(&self, tbt_data: XTPTickByTickStruct) {
        println!("Tick by tick: {:?}", tbt_data);
    }

    fn on_order_book(&self, ob: OrderBookStruct) {
        println!("Orderbook: {:?}", ob);
    }
}
