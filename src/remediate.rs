use super::gather;
use std::default::Default;

impl Default for S3RemediateOptions {
    fn default() -> Self {
        S3RemediateOptions {
            encryptobjects: true,
            trackobjects: true,
            skipwebbuckets: true,
            applylifecycle: true,
            applysseencryption: true,
            applytransitpolicy: false,
            customtransitpolicy: false,
            applykmskey: true,
        }
    }
}


pub struct S3RemediateOptions {
    encryptobjects: bool,
    trackobjects: bool,
    skipwebbuckets: bool,
    applylifecycle: bool,
    applysseencryption: bool,
    applytransitpolicy: bool,
    customtransitpolicy: bool,
    applykmskey: bool,
  }




pub fn remediate_buckets( remedy:S3RemediateOptions ){
    for b in super::gather::BUCKET_LIST.lock().unwrap().values(){
      //case skip web buckets
      println!("Test {}",b);
    }
}


