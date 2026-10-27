#![cfg(feature = "ely")]

use std::net::TcpListener;

use rmlib::extra::ely::start_beta_proxy;

fn pick_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

#[test]
fn start_beta_proxy_on_busy_port_returns_none() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let guard = start_beta_proxy(port);
    assert!(guard.is_none(), "expected None when port is busy");

    drop(listener);
}

#[test]
fn start_beta_proxy_on_free_port_returns_guard() {
    let port = pick_free_port();
    let guard = start_beta_proxy(port);
    assert!(guard.is_some(), "expected Some on free port");
    drop(guard);
}

#[test]
fn proxy_guard_drop_does_not_panic() {
    let port = pick_free_port();
    if let Some(guard) = start_beta_proxy(port) {
        drop(guard);
    }
}
