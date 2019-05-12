use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProduceRequest {
    pub topic: String,
    pub message: String,
}

impl fmt::Display for ProduceRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(topic: {}, message: {})", self.topic, self.message)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProduceResponse {
    pub result: String
}