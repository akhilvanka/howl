//External Crates
extern crate tree_magic;
extern crate keyring;
use serde::{Deserialize, Serialize};
use ureq;
extern crate clipboard;

//Use
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use read_input::prelude::*;

//Structure for Json Output
#[derive(Serialize, Deserialize)]
struct Output {
    url: String,
    shortName: String,
    originalName: String,
}

//One Function
fn main() -> std::io::Result<()> {
  //Set up the keyring service for the token
  let service = "howl";
  let username = "Bearer";
  let keyring = keyring::Keyring::new(&service, &username);
  let mut bearer = "Bearer ".to_owned();
  let _password = match keyring.get_password() {
      Ok(key) => {
       bearer = key;
      },
      Err(_er) => {
        let token: String = input().msg("You dont have a token currently set, lets add it:").get();
        bearer.push_str(&token);
        keyring.set_password(&bearer).ok();
     }
  };
  //Take the Arguments and make it a valid path
  let path = std::env::args().nth(1).expect("no path given");
  let srcdir = PathBuf::from(path.to_string());
  let n = fs::canonicalize(&srcdir).unwrap();
  //Open the file, read the meta data and its file name
  let f = File::open(&n).expect("This Borked");
  let filename = n.file_name().unwrap();
  let metadata = f.metadata()?;
  let buffered_reader = BufReader::new(f);
  //Read the mimetype of the fIle
  let result = tree_magic::from_filepath(&n);
  //POST using ureq
  let resp = ureq::post("https://pat.doggo.ninja/v1/upload")
      .set("Content-Type", "application/octet-stream")
      .set("Authorization", bearer.as_str())
      .set("Content-Length", &metadata.len().to_string())
      .query("originalName", filename.to_str().unwrap())
      .query("mimeType", &result)
      .send(buffered_reader);
  //Final Check and Json Parsing
  if resp.ok() {
    let data = resp.into_string().unwrap();
    let p: Output = serde_json::from_str(&data.as_str())?;
    println!("{}", p.url);
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(p.url.to_owned()).unwrap();
 } else {
    println!("error {}: {}", resp.status(), resp.into_string()?);
  }
  Ok(())
}
