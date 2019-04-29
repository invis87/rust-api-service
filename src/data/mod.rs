#[derive(Serialize, Deserialize, Debug)]
pub struct CalculateRequest {
    pub language: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CalculateResponse {
    pub result: String
}
