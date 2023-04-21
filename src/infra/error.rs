use anyhow::{anyhow, Error as AnyError};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

pub type Result<T, E = Status> = std::result::Result<T, E>;

/// From https://github.com/googleapis/googleapis/blob/master/google/rpc/error_details.proto
/// Design for error status compatibility
#[derive(Clone, Serialize)]
pub struct Status {
    code: Code,
    message: String,
    details: Option<Vec<ErrorDetail>>,
    #[serde(skip)]
    inner: Option<Arc<AnyError>>,
}

impl Status {
    pub fn new(
        code: Code,
        message: String,
        details: Option<Vec<ErrorDetail>>,
        inner: Option<Arc<AnyError>>,
    ) -> Self {
        Self {
            code,
            message,
            details,
            inner,
        }
    }

    pub fn ok() -> Self {
        Self::new(Code::Ok, "Ok.".to_string(), None, None)
    }

    pub fn cancelled() -> Self {
        Self::new(
            Code::Cancelled,
            "Request cancelled by the client.".to_string(),
            None,
            Some(Arc::new(anyhow!("Request cancelled by the client."))),
        )
    }

    pub fn unknown() -> Self {
        Self::new(
            Code::Unknown,
            "Unknown error.".to_string(),
            None,
            Some(Arc::new(anyhow!("Unknown error."))),
        )
    }

    pub fn invalid_argument(
        field: impl AsRef<str>,
        got: impl AsRef<str>,
        expect: impl AsRef<str>,
    ) -> Self {
        let msg = format!(
            "Request field '{}' is '{}', expected {}.",
            field.as_ref(),
            got.as_ref(),
            expect.as_ref()
        );
        Self::new(
            Code::InvalidArgument,
            msg.to_string(),
            None,
            Some(Arc::new(anyhow!(msg))),
        )
    }

    pub fn deadline_exceeded() -> Self {
        Self::new(
            Code::DeadlineExceeded,
            "Gateway timeout.".to_string(), // for client, it means gateway timeout.
            None,
            Some(Arc::new(anyhow!("Gateway timeout."))),
        )
    }

    pub fn not_found(resource: impl AsRef<str>) -> Self {
        let msg = format!("Resource '{}' not found.", resource.as_ref());
        Self::new(
            Code::NotFound,
            msg.to_string(),
            None,
            Some(Arc::new(anyhow!(msg))),
        )
    }

    pub fn already_exists(resource: impl AsRef<str>) -> Self {
        let msg = format!("Resource '{}' already exists.", resource.as_ref());
        Self::new(
            Code::AlreadyExists,
            msg.to_string(),
            None,
            Some(Arc::new(anyhow!(msg))),
        )
    }

    pub fn permission_denied(permission: impl AsRef<str>, resource: impl AsRef<str>) -> Self {
        let msg = format!(
            "Permission '{}' denied on resource '{}'.",
            permission.as_ref(),
            resource.as_ref()
        );
        Self::new(
            Code::PermissionDenied,
            msg.to_string(),
            None,
            Some(Arc::new(anyhow!(msg))),
        )
    }

    pub fn resource_exhausted() -> Self {
        Self::new(
            Code::ResourceExhausted,
            "Too many requests.".to_string(), // for client, it means too many requests.
            None,
            Some(Arc::new(anyhow!("Too many requests."))),
        )
    }

    pub fn failed_precondition() -> Self {
        Self::new(
            Code::FailedPrecondition,
            "Operation failed.".to_string(),
            None,
            Some(Arc::new(anyhow!("Operation failed."))),
        )
    }

    pub fn aborted() -> Self {
        Self::new(
            Code::Aborted,
            "Request aborted.".to_string(),
            None,
            Some(Arc::new(anyhow!("Request aborted."))),
        )
    }

    pub fn out_of_range<T: Debug>(field: impl AsRef<str>, range: Range<T>) -> Self {
        let msg = format!(
            "Parameter '{}' is out of range {}.",
            field.as_ref(),
            range.to_string()
        );
        Self::new(
            Code::OutOfRange,
            msg.to_string(),
            None,
            Some(Arc::new(anyhow!(msg))),
        )
    }

    pub fn unimplemented() -> Self {
        Self::new(
            Code::Unimplemented,
            "Not implemented.".to_string(),
            None,
            Some(Arc::new(anyhow!("Not implemented."))),
        )
    }

    pub fn internal() -> Self {
        Self::new(
            Code::Internal,
            "Internal error.".to_string(),
            None,
            Some(Arc::new(anyhow!("Internal error."))),
        )
    }

    pub fn unavailable() -> Self {
        Self::new(
            Code::Unavailable,
            "Service Unavailable.".to_string(),
            None,
            Some(Arc::new(anyhow!("Service Unavailable."))),
        )
    }

    pub fn data_loss() -> Self {
        Self::new(
            Code::DataLoss,
            "Internal error.".to_string(),
            None,
            Some(Arc::new(anyhow!("Internal error."))),
        )
    }

    pub fn unauthenticated() -> Self {
        Self::new(
            Code::Unauthenticated,
            "Invalid authentication credentials.".to_string(),
            None,
            Some(Arc::new(anyhow!("Invalid authentication credentials."))),
        )
    }

    #[inline]
    fn insert_detail(mut self, detail: ErrorDetail) -> Self {
        if let Some(details) = &mut self.details {
            details.push(detail);
            return self;
        }
        self.details = Some(vec![detail]);
        self
    }

    pub fn with_error_info(
        self,
        reason: impl Into<String>,
        domain: impl Into<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        self.insert_detail(ErrorDetail::ErrorInfo {
            reason: reason.into(),
            domain: domain.into(),
            metadata,
        })
    }

    pub fn with_retry_info(self, delay: Duration) -> Self {
        self.insert_detail(ErrorDetail::RetryInfo { retry_delay: delay })
    }

    pub fn with_debug_info(self, capture_stack: bool, detail: impl Into<String>) -> Self {
        let backtrace = Backtrace::capture();
        let frames = backtrace.frames();
        self.insert_detail(ErrorDetail::DebugInfo {
            stack_entries: if capture_stack {
                Some(
                    frames
                        .iter()
                        .map(|frame| format!("{:?}", frame))
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            },
            detail: detail.into(),
        })
    }

    pub fn with_quota(self, vio: impl Into<Vec<QuotaViolation>>) -> Self {
        self.insert_detail(ErrorDetail::QuotaFailure {
            violations: vio.into(),
        })
    }

    pub fn with_precondition(self, vio: impl Into<Vec<PreconditionViolation>>) -> Self {
        self.insert_detail(ErrorDetail::PreconditionFailure {
            violations: vio.into(),
        })
    }

    pub fn with_bad_request(self, vio: impl Into<Vec<FieldViolation>>) -> Self {
        self.insert_detail(ErrorDetail::BadRequest {
            field_violations: vio.into(),
        })
    }

    pub fn with_request_info(
        self,
        request_id: impl Into<String>,
        serving_data: impl Into<String>,
    ) -> Self {
        self.insert_detail(ErrorDetail::RequestInfo {
            request_id: request_id.into(),
            serving_data: serving_data.into(),
        })
    }

    pub fn with_resource_info(
        self,
        resource_type: impl Into<String>,
        resource_name: impl Into<String>,
        owner: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.insert_detail(ErrorDetail::ResourceInfo {
            resource_type: resource_type.into(),
            resource_name: resource_name.into(),
            owner: owner.into(),
            description: description.into(),
        })
    }

    pub fn with_help(self, links: impl Into<Vec<Link>>) -> Self {
        self.insert_detail(ErrorDetail::Help {
            links: links.into(),
        })
    }

    pub fn with_localized(self, locale: impl Into<String>, message: impl Into<String>) -> Self {
        self.insert_detail(ErrorDetail::LocalizedMessage {
            locale: locale.into(),
            message: message.into(),
        })
    }
}

pub enum Range<T> {
    Discrete(Vec<T>),
    Continuous(T, T),
}

impl<T: Debug> ToString for Range<T> {
    fn to_string(&self) -> String {
        match self {
            Range::Discrete(v) => format!("{:?}", v),
            Range::Continuous(low, high) => format!("[{:?}, {:?}]", low, high),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum Code {
    /// The operation completed successfully.
    Ok = 0,

    /// The operation was cancelled.
    Cancelled = 1,

    /// Unknown error.
    Unknown = 2,

    /// Client specified an invalid argument.
    InvalidArgument = 3,

    /// Deadline expired before operation could complete.
    DeadlineExceeded = 4,

    /// Some requested entity was not found.
    NotFound = 5,

    /// Some entity that we attempted to create already exists.
    AlreadyExists = 6,

    /// The caller does not have permission to execute the specified operation.
    PermissionDenied = 7,

    /// Some resource has been exhausted.
    ResourceExhausted = 8,

    /// The system is not in a state required for the operation's execution.
    FailedPrecondition = 9,

    /// The operation was aborted.
    Aborted = 10,

    /// Operation was attempted past the valid range.
    OutOfRange = 11,

    /// Operation is not implemented or not supported.
    Unimplemented = 12,

    /// Internal error.
    Internal = 13,

    /// The service is currently unavailable.
    Unavailable = 14,

    /// Unrecoverable data loss or corruption.
    DataLoss = 15,

    /// The request does not have valid authentication credentials
    Unauthenticated = 16,
}

impl Code {
    /// If you only need description in `println`, `format`, `log` and other
    /// formatting contexts, you may want to use `Display` impl for `Code`
    /// instead.
    pub fn description(&self) -> &'static str {
        match self {
            Code::Ok => "The operation completed successfully",
            Code::Cancelled => "The operation was cancelled",
            Code::Unknown => "Unknown error",
            Code::InvalidArgument => "Client specified an invalid argument",
            Code::DeadlineExceeded => "Deadline expired before operation could complete",
            Code::NotFound => "Some requested entity was not found",
            Code::AlreadyExists => "Some entity that we attempted to create already exists",
            Code::PermissionDenied => {
                "The caller does not have permission to execute the specified operation"
            }
            Code::ResourceExhausted => "Some resource has been exhausted",
            Code::FailedPrecondition => {
                "The system is not in a state required for the operation's execution"
            }
            Code::Aborted => "The operation was aborted",
            Code::OutOfRange => "Operation was attempted past the valid range",
            Code::Unimplemented => "Operation is not implemented or not supported",
            Code::Internal => "Internal error",
            Code::Unavailable => "The service is currently unavailable",
            Code::DataLoss => "Unrecoverable data loss or corruption",
            Code::Unauthenticated => "The request does not have valid authentication credentials",
        }
    }

    pub fn to_http_code(&self) -> StatusCode {
        match self {
            Code::Ok => StatusCode::OK,
            Code::Cancelled => StatusCode::from_u16(499).unwrap(),
            Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Code::InvalidArgument => StatusCode::BAD_REQUEST,
            Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
            Code::NotFound => StatusCode::NOT_FOUND,
            Code::AlreadyExists => StatusCode::CONFLICT,
            Code::PermissionDenied => StatusCode::FORBIDDEN,
            Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            Code::FailedPrecondition => StatusCode::BAD_REQUEST,
            Code::Aborted => StatusCode::CONFLICT,
            Code::OutOfRange => StatusCode::BAD_REQUEST,
            Code::Unimplemented => StatusCode::NOT_IMPLEMENTED,
            Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Code::DataLoss => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.description(), f)
    }
}

impl Debug for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Error");

        builder.field("code", &self.code);

        if !self.message.is_empty() {
            builder.field("message", &self.message);
        }

        if let Some(details) = &self.details {
            builder.field("details", &details);
        }

        if let Some(inner) = &self.inner {
            builder.field("source", &inner.source());
        }

        builder.finish()
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error: {}, message: {}, details: {}",
            self.code,
            self.message,
            serde_json::to_string(&self.details).map_err(|_| std::fmt::Error)?,
        )
    }
}

impl<E> From<E> for Status
where
    E: StdError + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Self {
            code: Code::Unknown, // The indication of an error from any source is unknown.
            message: "Unknown error.".to_string(),
            details: None,
            inner: Some(Arc::new(AnyError::from(err))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum ErrorDetail {
    ErrorInfo {
        reason: String,
        domain: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, String>>, // String is an interior mutable type, it's not suitable to use it as the Hash key
    },
    RetryInfo {
        retry_delay: Duration,
    },
    DebugInfo {
        #[serde(skip_serializing_if = "Option::is_none")]
        stack_entries: Option<Vec<String>>,
        detail: String,
    },
    QuotaFailure {
        violations: Vec<QuotaViolation>,
    },
    PreconditionFailure {
        violations: Vec<PreconditionViolation>,
    },
    BadRequest {
        field_violations: Vec<FieldViolation>,
    },
    RequestInfo {
        request_id: String,
        serving_data: String,
    },
    ResourceInfo {
        resource_type: String,
        resource_name: String,
        owner: String,
        description: String,
    },
    Help {
        links: Vec<Link>,
    },
    LocalizedMessage {
        locale: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaViolation {
    pub subject: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreconditionViolation {
    pub r#type: String,
    pub subject: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldViolation {
    pub filed: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub description: String,
    pub url: String,
}

macro_rules! is_implement {
    ( $(
        ( $lower:ident, $upper:ident );
    )+ ) => {
        $(
            pub fn $lower(&self) -> bool {
                matches!(self, Self::$upper { .. })
            }
        )*
    };
}

impl ErrorDetail {
    is_implement!(
        (is_error_info, ErrorInfo);
        (is_retry_info, RetryInfo);
        (is_debug_info, DebugInfo);
        (is_quota_failure, QuotaFailure);
        (is_precondition_failure, PreconditionFailure);
        (is_bad_request, BadRequest);
        (is_request_info, RequestInfo);
        (is_resource_info, ResourceInfo);
        (is_help, Help);
        (is_localized_message, LocalizedMessage);
    );

    /// Debug info is sensitive and it's details should not be exposed to API
    pub fn is_sensitive(&self) -> bool {
        self.is_debug_info()
    }
}
