use esp_idf_svc::{http::{self, client::{Configuration, EspHttpConnection}, Method}, sys::EspError};

#[derive(Debug)]
pub enum HttpError {
    InizializationError,
    RequestError,
    ListeningError,
    TimeoutError,
    ReadError
}

pub trait Http {

    fn new() -> Result<Self, HttpError> where Self: Sized;

    fn get_connection(&mut self) -> &mut EspHttpConnection;

    fn post<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        let temp: Vec<(&'a str, &'a str)> = headers.iter().map(|header| (header.header_type.to_string(), header.value)).collect();
        self.get_connection().initiate_request(Method::Post, uri, &temp).map_err(|_| HttpError::RequestError)
    }

    fn get<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        let temp: Vec<(&'a str, &'a str)> = headers.iter().map(|header| (header.header_type.to_string(), header.value)).collect();
        self.get_connection().initiate_request(Method::Get, uri, &temp).map_err(|_| HttpError::RequestError)
    }

    fn put<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        let temp: Vec<(&'a str, &'a str)> = headers.iter().map(|header| (header.header_type.to_string(), header.value)).collect();
        self.get_connection().initiate_request(Method::Put, uri, &temp).map_err(|_| HttpError::RequestError)
    }

    fn delete<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        let temp: Vec<(&'a str, &'a str)> = headers.iter().map(|header| (header.header_type.to_string(), header.value)).collect();
        self.get_connection().initiate_request(Method::Delete, uri, &temp).map_err(|_| HttpError::RequestError)
    }
    
    fn patch<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        let temp: Vec<(&'a str, &'a str)> = headers.iter().map(|header| (header.header_type.to_string(), header.value)).collect();
        self.get_connection().initiate_request(Method::Patch, uri, &temp).map_err(|_| HttpError::RequestError)
    }

    fn head<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        let temp: Vec<(&'a str, &'a str)> = headers.iter().map(|header| (header.header_type.to_string(), header.value)).collect();
        self.get_connection().initiate_request(Method::Head, uri, &temp).map_err(|_| HttpError::RequestError)
    }

    fn options<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        let temp: Vec<(&'a str, &'a str)> = headers.iter().map(|header| (header.header_type.to_string(), header.value)).collect();
        self.get_connection().initiate_request(Method::Options, uri, &temp).map_err(|_| HttpError::RequestError)
    }

    fn response_status(&mut self) -> u16 {
        self.get_connection().status()
    }
  
    fn response_status_message(&mut self) -> Option<&str> {
        self.get_connection().status_message() // TODO: This can panic. Add it to the docu.
    }

    fn wait_for_response(&mut self, buffer: &mut [u8]) -> Result<usize, HttpError>{
        self.get_connection().initiate_response().map_err(|_| HttpError::ListeningError)?;
        self.get_connection().read(buffer).map_err(|err| 
            match err.code() {
                -0x7007 => HttpError::TimeoutError,
                _ => HttpError::ReadError
            }
        )
    }  
    
}

pub struct HttpClient{
    connection: EspHttpConnection,
}

impl Http for HttpClient {
    fn new() -> Result<Self, HttpError> {
        let config: &Configuration = &Default::default();
        let connection = EspHttpConnection::new(config).map_err(|_| HttpError::InizializationError)?;
        Ok( HttpClient { connection } )
    }

    fn get_connection(&mut self) -> &mut EspHttpConnection {
        &mut self.connection
    }
}

pub struct HttpsClient {
    connection: EspHttpConnection
}

impl Http for HttpsClient {
    fn new() -> Result<Self, HttpError> where Self: Sized {
        let config: &Configuration = &Configuration{
            use_global_ca_store: true,
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            ..Default::default()
        };
        let connection = EspHttpConnection::new(config).map_err(|_| HttpError::InizializationError)?;
        Ok( HttpsClient { connection } )
    }

    fn get_connection(&mut self) -> &mut EspHttpConnection {
        &mut self.connection
    }
}

pub struct HttpHeader<'a> {
    header_type: HttpHeaderType<'a>,
    value: &'a str
}

impl<'a> HttpHeader<'a> {
    pub fn new(header_type: HttpHeaderType<'a>, value: &'a str) -> Self {
        HttpHeader{header_type,value}
    }
}


pub enum HttpHeaderType<'a> {
    AIM,
    Accept,
    AcceptCharset,
    AcceptDatetime,
    AcceptEncoding,
    AcceptLanguage,
    AccessControlRequestMethod,
    Authorization,
    CacheControl,
    Connection,
    ContentEncoding,
    ContentLength,
    ContentMD5,
    ContentType,
    Cookie,
    Custom( &'a str ),
    Date,
    Expect,
    Forwarded,
    From,
    Host,
    HTTP2Settings,
    IfMatch,
    IfModifiedSince,
    IfNoneMatch,
    IfRange,
    IfUnmodifiedSince,
    MaxForwards,
    Origin,
    Pragma,	
    Prefer,	
    ProxyAuthorization,
    Range,	
    Referer,
    TE,
    Trailer,
    TransferEncoding,
    UserAgent,
    Upgrade,
    Via,
    Warning,
}

impl<'a> HttpHeaderType<'a> {
    fn to_string(&self) -> &'a str {
        match self{
            HttpHeaderType::AIM => "A-IM",
            HttpHeaderType::Accept => "Accept",
            HttpHeaderType::AcceptCharset => "Accept-Charset",
            HttpHeaderType::AcceptDatetime => "Accept-Datetime",
            HttpHeaderType::AcceptEncoding => "Accept-Encoding",
            HttpHeaderType::AcceptLanguage => "Accept-Language",
            HttpHeaderType::AccessControlRequestMethod => "Access-Control-Request-Method",
            HttpHeaderType::Authorization => "Authorization",
            HttpHeaderType::CacheControl => "Cache-Control",
            HttpHeaderType::Connection => "Connection",
            HttpHeaderType::ContentEncoding => "Content-Encoding",
            HttpHeaderType::ContentLength => "Content-Length",
            HttpHeaderType::ContentMD5 => "Content-MD5",
            HttpHeaderType::ContentType => "Content-Type",
            HttpHeaderType::Cookie => "Cookie",
            HttpHeaderType::Date => "Date",
            HttpHeaderType::Expect => "Expect",
            HttpHeaderType::Forwarded => "Forwarded",
            HttpHeaderType::From => "From",
            HttpHeaderType::Host => "Host",
            HttpHeaderType::HTTP2Settings => "HTTP2-Settings",
            HttpHeaderType::IfMatch => "If-Match",
            HttpHeaderType::IfModifiedSince => "If-Modified-Since",
            HttpHeaderType::IfNoneMatch => "If-None-Match",
            HttpHeaderType::IfRange => "If-Range",
            HttpHeaderType::IfUnmodifiedSince => "If-Unmodified-Since",
            HttpHeaderType::MaxForwards => "Max-Forwards",
            HttpHeaderType::Origin => "Origin",
            HttpHeaderType::Pragma => "Pragma",
            HttpHeaderType::Prefer => "Prefer",
            HttpHeaderType::ProxyAuthorization => "Proxy-Authorization",
            HttpHeaderType::Range => "Range",
            HttpHeaderType::Referer => "Referer",
            HttpHeaderType::TE => "TE",
            HttpHeaderType::Trailer => "Trailer",
            HttpHeaderType::TransferEncoding => "Transfer-Encoding",
            HttpHeaderType::UserAgent => "User-Agent",
            HttpHeaderType::Upgrade => "Upgrade",
            HttpHeaderType::Via => "Via",
            HttpHeaderType::Warning => "Warning",
            HttpHeaderType::Custom(h_type) => h_type,
        }        
    }

}
