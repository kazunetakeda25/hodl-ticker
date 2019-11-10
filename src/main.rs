mod cell;
mod crypto;
mod currency;
mod helper;
mod layout;
mod provider;

use crate::currency::Currency;
use crate::layout::Layout;
use clap::{load_yaml, App};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion;

const DEFAULT_CURRENCY: &str = "usd";
const SLEEP_DURATION: u64 = 20;
// Limit our results to 10 crypto
const ENDPOINT: &str = "https://api.coinmarketcap.com/v1/ticker";

fn make_uri(currency: &Currency, limit: &str) -> String {
    format!(
        "{}?limit={}&convert={}",
        ENDPOINT,
        limit,
        currency.get_name()
    )
}

fn clear_screen() {
    print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
}

fn erase_screen() {
    print!(
        "{}{}",
        termion::clear::AfterCursor,
        termion::cursor::Goto(1, 1)
    );
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let limit: &str = matches.value_of("limit").unwrap_or("10");
    let currency =
        Currency::new(matches.value_of("convert").unwrap_or(DEFAULT_CURRENCY));
    let uri = make_uri(&currency, limit);
    let (tx, rx) = mpsc::channel();

    let mut filter_list = vec![];
    if let Some(list) = matches.values_of("filter") {
        filter_list = list.collect();
    }

    // Clear terminal screen
    clear_screen();

    if !matches.is_present("watch") {
        let data = provider::get(&uri.clone()).unwrap_or_else(|_| vec![]);
        let layout = Layout::new(data, filter_list.clone(), currency.clone());
        return layout.print();
    }

    thread::spawn(move || loop {
        let data = provider::get(&uri.clone()).unwrap_or_else(|_| vec![]);
        tx.send(data).unwrap();
        // Every 5 sec
        thread::sleep(Duration::new(SLEEP_DURATION, 0));
    });

    for data in rx {
        erase_screen();
        let layout = Layout::new(data, filter_list.clone(), currency.clone());
        layout.print();
    }
}
