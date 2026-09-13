#[derive(Clone, Debug)]
pub struct NativeRequest {
    pub baseUrl : String,
    pub path : String,
    pub method : String,
    pub isMultipart : bool,
    pub requestType : String,
    pub responseType : String,
    pub body :RequestBody
}

#[derive(Clone, Debug)]
pub struct RequestBody{
    pub model : String,
    pub prompt : String,
    pub options : String,
    pub keepAlive : String,
}