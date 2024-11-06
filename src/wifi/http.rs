use esp_idf_svc::http::{
    client::{Configuration, EspHttpConnection},
    Method,
};

#[derive(Debug)]
pub enum HttpError {
    InizializationError,
    ListeningError,
    ReadError,
    RequestError,
    TimeoutError,
}

/// The Http trait gives the implementation on how to do the basic HTTP methods and wait for
/// their response
pub trait Http {
    fn new() -> Result<Self, HttpError>
    where
        Self: Sized;

    /// Returns the EspHttpConnection
    fn get_connection(&mut self) -> &mut EspHttpConnection;

    /// Checks whether the "Content-Length" header are in the headers section, if not it adds it to them.
    ///
    /// # Arguments
    ///
    /// - `headers`: The mutable reference to a vectorof headers the user added to the HTTP request.
    /// - `body_len`: An option `usize` may contain the lenght of the request body if the user set one.
    fn add_body_len_header(&self, headers: &mut Vec<HttpHeader>, body_len: Option<usize>) {
        if let Some(body_len) = body_len {
            let has_content_length = headers
                .iter()
                .any(|header| header.header_type == HttpHeaderType::ContentLength);

            if !has_content_length {
                let content_length_header =
                    HttpHeader::new(HttpHeaderType::ContentLength, body_len.to_string());
                headers.push(content_length_header);
            }
        }
    }

    /// Sends an HTTP request to a specified URI with the given method, headers, and optional body.
    ///
    /// # Parameters
    /// - `method`: The HTTP method to use for the request (e.g., GET, POST).
    /// - `uri`: A string slice that represents the URI to which the request will be sent.
    /// - `headers`: A vector of HTTP headers to include with the request.
    /// - `body`: An optional `String` containing the body of the request. If `None`, no body is sent.
    ///
    /// # Returns
    /// Returns a `Result<(), HttpError>`. On success, it returns `Ok(())`, HttpError otherwise.
    ///
    /// # Errors
    /// - `HttpError::RequestError`: If an error occurs in while creating or sending the request.
    fn send_request(
        &mut self,
        method: Method,
        uri: &str,
        mut headers: Vec<HttpHeader>,
        body: Option<String>,
    ) -> Result<(), HttpError> {
        self.add_body_len_header(&mut headers, body.as_ref().map(|body| body.len()));

        let temp: Vec<(&str, &str)> = headers
            .iter()
            .map(|header| (header.header_type.to_string(), header.value.as_str()))
            .collect();
        let connection = self.get_connection();
        connection
            .initiate_request(method, uri, &temp)
            .map_err(|_| HttpError::RequestError)?;
        if let Some(body_content) = body {
            println!("{:?}", body_content.clone());
            connection
                .write_all(body_content.as_bytes())
                .map_err(|_| HttpError::RequestError)?;
        }
        Ok(())
    }

    /// Does an HTTP POST on the desired uri with the designated headers
    ///
    /// # Arguments
    ///
    /// - `uri`: A string slice that holds the Uniform Resource Identifier (URI) of the target resource where the HTTP POST request will be sent.
    /// - `headers`: A vector of HttpHeader structs containing the headers to be included in the POST request.
    /// - `body`: An optional `String` containing the body of the request. If `None`, no body is sent.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the POST operation completed successfully, or an `HttpError` if it fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::RequestError`: If the request fails.
    fn post<'a>(
        &mut self,
        uri: &'a str,
        headers: Vec<HttpHeader<'a>>,
        body: Option<String>,
    ) -> Result<(), HttpError> {
        self.send_request(Method::Post, uri, headers, body)
    }

    /// Does an HTTP GET on the desired uri with the designated headers
    ///
    /// # Arguments
    ///
    /// - `uri`: A string slice that holds the Uniform Resource Identifier (URI) of the target resource where the HTTP GET request will be sent.
    /// - `headers`: A vector of HttpHeader structs containing the headers to be included in the GET request.
    /// - `body`: An optional `String` containing the body of the request. If `None`, no body is sent.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the GET operation completed successfully, or an `HttpError` if it fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::RequestError`: If the request fails.
    fn get<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        self.send_request(Method::Get, uri, headers, None)
    }

    /// Does an HTTP PUT on the desired uri with the designated headers
    ///
    /// # Arguments
    ///
    /// - `uri`: A string slice that holds the Uniform Resource Identifier (URI) of the target resource where the HTTP PUT request will be sent.
    /// - `headers`: A vector of HttpHeader structs containing the headers to be included in the PUT request.
    /// - `body`: An optional `String` containing the body of the request. If `None`, no body is sent.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the PUT operation completed successfully, or an `HttpError` if it fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::RequestError`: If the request fails.
    fn put<'a>(
        &mut self,
        uri: &'a str,
        headers: Vec<HttpHeader<'a>>,
        body: Option<String>,
    ) -> Result<(), HttpError> {
        self.send_request(Method::Put, uri, headers, body)
    }

    /// Does an HTTP DELETE on the desired uri with the designated headers
    ///
    /// # Arguments
    ///
    /// - `uri`: A string slice that holds the Uniform Resource Identifier (URI) of the target resource where the HTTP DELETE request will be sent.
    /// - `headers`: A vector of HttpHeader structs containing the headers to be included in the DELETE request.
    /// - `body`: An optional `String` containing the body of the request. If `None`, no body is sent.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the DELETE operation completed successfully, or an `HttpError` if it fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::RequestError`: If the request fails.
    fn delete<'a>(
        &mut self,
        uri: &'a str,
        headers: Vec<HttpHeader<'a>>,
        body: Option<String>,
    ) -> Result<(), HttpError> {
        self.send_request(Method::Delete, uri, headers, body)
    }

    /// Does an HTTP PATCH on the desired uri with the designated headers
    ///
    /// # Arguments
    ///
    /// - `uri`: A string slice that holds the Uniform Resource Identifier (URI) of the target resource where the HTTP PATCH request will be sent.
    /// - `headers`: A vector of HttpHeader structs containing the headers to be included in the PATCH request.
    /// - `body`: An optional `String` containing the body of the request. If `None`, no body is sent.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the PATCH operation completed successfully, or an `HttpError` if it fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::RequestError`: If the request fails.
    fn patch<'a>(
        &mut self,
        uri: &'a str,
        headers: Vec<HttpHeader<'a>>,
        body: Option<String>,
    ) -> Result<(), HttpError> {
        self.send_request(Method::Patch, uri, headers, body)
    }

    /// Does an HTTP HEAD on the desired uri with the designated headers
    ///
    /// # Arguments
    ///
    /// - `uri`: A string slice that holds the Uniform Resource Identifier (URI) of the target resource where the HTTP HEAD request will be sent.
    /// - `headers`: A vector of HttpHeader structs containing the headers to be included in the HEAD request.
    /// - `body`: An optional `String` containing the body of the request. If `None`, no body is sent.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the HEAD operation completed successfully, or an `HttpError` if it fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::RequestError`: If the request fails.
    fn head<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        self.send_request(Method::Head, uri, headers, None)
    }

    /// Does an HTTP OPTIONS on the desired uri with the designated headers
    ///
    /// # Arguments
    ///
    /// - `uri`: A string slice that holds the Uniform Resource Identifier (URI) of the target resource where the HTTP OPTIONS request will be sent.
    /// - `headers`: A vector of HttpHeader structs containing the headers to be included in the OPTIONS request.
    /// - `body`: An optional `String` containing the body of the request. If `None`, no body is sent.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the OPTIONS operation completed successfully, or an `HttpError` if it fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::RequestError`: If the request fails.
    fn options<'a>(&mut self, uri: &'a str, headers: Vec<HttpHeader<'a>>) -> Result<(), HttpError> {
        self.send_request(Method::Options, uri, headers, None)
    }

    /// Gets the response status code of the last done request
    ///
    /// # Returns
    ///
    /// An u16 that represents the status code
    fn response_status(&mut self) -> u16 {
        self.get_connection().status()
    }

    /// Gets the response status message of the last done request
    ///
    /// # Returns
    ///
    /// An Option. A Some with an &str if there was a status message to get. Otherwise a None.
    ///
    /// # Panics
    ///
    /// If connection is not in response phase
    fn response_status_message(&mut self) -> Option<&str> {
        self.get_connection().status_message()
    }

    /// Blocking wait of the request response
    ///
    /// # Arguments
    ///
    /// - `buffer`: A slice of bytes used to store the response
    ///
    /// # Returns
    ///
    /// A Result. An Ok with an usize representing the bytes read if operation was succesful.
    /// Otherwise an `HttpError` if it fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::ListeningError`: If initiating the response phase fails.
    /// - `HttpError::TimeoutError`: If there is a timeout waiting for the response.
    /// - `HttpError::ReadError`: If the reading operation fails.
    fn wait_for_response(&mut self, buffer: &mut [u8]) -> Result<usize, HttpError> {
        self.get_connection()
            .initiate_response()
            .map_err(|_| HttpError::ListeningError)?;
        self.get_connection()
            .read(buffer)
            .map_err(|err| match err.code() {
                -0x7007 => HttpError::TimeoutError,
                _ => HttpError::ReadError,
            })
    }
}

/// Abstraction to simply make HTTP request as a client
pub struct HttpClient {
    connection: EspHttpConnection,
}

impl Http for HttpClient {
    /// Creates a new HttpClient
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `HttpClient` instance, or an `HttpError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::InizializationError`: If the creation of the Http connection fails
    fn new() -> Result<Self, HttpError> {
        let config: &Configuration = &Default::default();
        let connection =
            EspHttpConnection::new(config).map_err(|_| HttpError::InizializationError)?;
        Ok(HttpClient { connection })
    }

    /// Gets the EspHttpConnection
    ///
    /// # Returns
    ///
    /// A mutable reference of the EspHttpConnection
    fn get_connection(&mut self) -> &mut EspHttpConnection {
        &mut self.connection
    }
}

/// Abstraction to simply make HTTPS request as a client
pub struct HttpsClient {
    connection: EspHttpConnection,
}

impl Http for HttpsClient {
    /// Creates a new HttpsClient
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `HttpsClient` instance, or an `HttpError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `HttpError::InizializationError`: If the creation of the Http connection fails
    fn new() -> Result<Self, HttpError>
    where
        Self: Sized,
    {
        let config: &Configuration = &Configuration {
            use_global_ca_store: true,
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            ..Default::default()
        };
        let connection =
            EspHttpConnection::new(config).map_err(|_| HttpError::InizializationError)?;
        Ok(HttpsClient { connection })
    }

    /// Gets the EspHttpConnection
    ///
    /// # Returns
    ///
    /// A mutable reference of the EspHttpConnection
    fn get_connection(&mut self) -> &mut EspHttpConnection {
        &mut self.connection
    }
}

/// Simple abstraction of a header used for HTTP/HTTPS requests. It contains:
/// - `header_type`: The tyep of header to be used
/// - `value`: The value associated to the header
#[derive(Debug)]
pub struct HttpHeader<'a> {
    header_type: HttpHeaderType<'a>,
    value: String,
}

impl<'a> HttpHeader<'a> {
    /// Creates a new HttpHeaderType
    ///
    /// # Arguments
    ///
    /// - `header_type`: The tyep of header to be used
    /// - `value`: The value associated to the header
    ///
    /// # Returns
    ///
    /// The new HttpHeader instance
    pub fn new(header_type: HttpHeaderType<'a>, value: String) -> Self {
        HttpHeader { header_type, value }
    }
}

/// Standard HTTP/HTTPS headers
#[derive(Debug, PartialEq, Eq)]
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
    Custom(&'a str),
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
    /// Creates the &str for the enum instance
    ///
    /// # Returns
    ///
    /// An &str of the header type
    fn to_string(&self) -> &'a str {
        match self {
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
