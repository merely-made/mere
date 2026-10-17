fn main(){
 let a:Vec<String>=std::env::args().collect();
 if a.len()==6 && a[5]=="pending" {std::fs::write(&a[4],br#"{"result":"failed"}"#).unwrap();return;}
 if let Ok(d)=std::env::var("ESP_STREAMING_WORK_DIR"){std::fs::write(std::path::Path::new(&d).join("burn-wgpu-software.json"),b"{}").unwrap();}
 else if let Ok(d)=std::env::var("ESP_FORMAT_WORK_DIR"){let n=format!("{}-{}.json",std::env::var("ESP_WEIGHT_FORMAT").unwrap(),std::env::var("ESP_WEIGHT_LOADER").unwrap());std::fs::write(std::path::Path::new(&d).join(n),b"{}").unwrap();}
 println!("running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out");
}
