#![feature(generators, generator_trait)]

use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::ops::{Generator, GeneratorState};
use std::pin::Pin;
use std::process::Command;

use rofi_plugin::{generator, mode, select};
use serde::Deserialize;

#[derive(Deserialize)]
struct MullvadDestination {
	hostname: String,
	country_name: String,
	city_name: String,

	pubkey: String,
	multihop_port: u16,
	ipv4_addr_in: String,
}
impl Display for MullvadDestination {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{} - {} ({})",
			self.country_name,
			self.city_name,
			self.hostname.trim_end_matches("-wireguard")
		)
	}
}

generator!(GeneratorRofi, {
	let json: Vec<MullvadDestination> =
		serde_json::from_reader(File::open("./mullvad.json").unwrap()).unwrap();
	let names: Vec<_> = json.iter().map(|dest| dest.to_string()).collect();
	let (m, first) = select!(GeneratorAction::ReplaceItems(
		Some("Select first region:\nHold <b>shift</b> for second hop selection".into()),
		names.clone(),
	));
	let second = if m.shift {
		let (_, second) = select!(GeneratorAction::ReplaceItems(
			Some("Select destination for second hop:".into()),
			names
		));
		Some(second)
	} else {
		None
	};
	let host = &json[first].ipv4_addr_in;
	let port = second.map(|s| json[s].multihop_port).unwrap_or(51820);
	let pubkey = second
		.map(|s| &json[s].pubkey)
		.unwrap_or(&json[first].pubkey);

	let config = format!(
		r#"
            [Interface]
            PrivateKey = {}
            Address = {}

            [Peer]
            PublicKey = {}
            Endpoint = {}:{}
            AllowedIPs = 0.0.0.0/0, ::/0
        "#,
		std::env::var("PRIVATE_KEY").unwrap(),
		std::env::var("ADDRESS").unwrap(),
		pubkey,
		host,
		port,
	);
	let _down = Command::new("sudo")
		.arg("wg-quick")
		.arg("down")
		.arg(format!(
			"{}/mullvad.conf",
			std::env::current_dir().unwrap().to_str().unwrap()
		))
		.spawn()
		.unwrap()
		.wait()
		.unwrap();
	File::create("mullvad.conf")
		.unwrap()
		.write_all(config.as_bytes())
		.unwrap();
	Command::new("sudo")
		.arg("wg-quick")
		.arg("up")
		.arg(format!(
			"{}/mullvad.conf",
			std::env::current_dir().unwrap().to_str().unwrap()
		))
		.spawn()
		.unwrap()
		.wait()
		.unwrap();
});
mode!(GeneratorRofi, "mullvad", "Mullvad");
