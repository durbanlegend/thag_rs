#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;

pub fn tool_errors_impl(_tokens: TokenStream) -> TokenStream {
    quote::quote! {
    pub enum ToolError {
          // Thread-safe wrapper for generic errors
          ThreadSafe(Box<dyn std::error::Error + Send + Sync + 'static>),

          // Common standard library errors with direct variants
          Io(std::io::Error),
          Fmt(std::fmt::Error),
          ParseInt(std::num::ParseIntError),
          ParseFloat(std::num::ParseFloatError),
          Utf8(std::str::Utf8Error),
          FromUtf8(std::string::FromUtf8Error),

          // Simple message error
          Message(String),
      }

      // Implement From for Box<dyn Error + Send + Sync>
      impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for ToolError {
          fn from(err: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
              Self::ThreadSafe(err)
          }
      }

      // Implement From for common standard library errors
      impl From<std::io::Error> for ToolError {
          fn from(err: std::io::Error) -> Self {
              Self::Io(err)
          }
      }

      impl From<std::fmt::Error> for ToolError {
          fn from(err: std::fmt::Error) -> Self {
              Self::Fmt(err)
          }
      }

      impl From<std::num::ParseIntError> for ToolError {
          fn from(err: std::num::ParseIntError) -> Self {
              Self::ParseInt(err)
          }
      }

      impl From<std::num::ParseFloatError> for ToolError {
          fn from(err: std::num::ParseFloatError) -> Self {
              Self::ParseFloat(err)
          }
      }

      impl From<std::str::Utf8Error> for ToolError {
          fn from(err: std::str::Utf8Error) -> Self {
              Self::Utf8(err)
          }
      }

      impl From<std::string::FromUtf8Error> for ToolError {
          fn from(err: std::string::FromUtf8Error) -> Self {
              Self::FromUtf8(err)
          }
      }

      // Implement From for string types to easily create message errors
      impl From<String> for ToolError {
          fn from(msg: String) -> Self {
              Self::Message(msg)
          }
      }

      impl From<&str> for ToolError {
          fn from(msg: &str) -> Self {
              Self::Message(msg.to_string())
          }
      }

      // Implement std::error::Error trait
      impl std::error::Error for ToolError {
          fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
              match self {
                  Self::ThreadSafe(e) => Some(&**e),
                  Self::Io(e) => Some(e),
                  Self::Fmt(e) => Some(e),
                  Self::ParseInt(e) => Some(e),
                  Self::ParseFloat(e) => Some(e),
                  Self::Utf8(e) => Some(e),
                  Self::FromUtf8(e) => Some(e),
                  Self::Message(_) => None,
              }
          }
      }

      // Implement Display trait
      impl std::fmt::Display for ToolError {
          fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
              match self {
                  Self::ThreadSafe(e) => write!(f, "{}", e),
                  Self::Io(e) => write!(f, "IO error: {}", e),
                  Self::Fmt(e) => write!(f, "Format error: {}", e),
                  Self::ParseInt(e) => write!(f, "Integer parsing error: {}", e),
                  Self::ParseFloat(e) => write!(f, "Float parsing error: {}", e),
                  Self::Utf8(e) => write!(f, "UTF-8 error: {}", e),
                  Self::FromUtf8(e) => write!(f, "From UTF-8 error: {}", e),
                  Self::Message(msg) => write!(f, "{}", msg),
              }
          }
      }

      // Implement Debug trait
      impl std::fmt::Debug for ToolError {
          fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
              match self {
                  Self::ThreadSafe(e) => write!(f, "ThreadSafe({:?})", e),
                  Self::Io(e) => write!(f, "Io({:?})", e),
                  Self::Fmt(e) => write!(f, "Fmt({:?})", e),
                  Self::ParseInt(e) => write!(f, "ParseInt({:?})", e),
                  Self::ParseFloat(e) => write!(f, "ParseFloat({:?})", e),
                  Self::Utf8(e) => write!(f, "Utf8({:?})", e),
                  Self::FromUtf8(e) => write!(f, "FromUtf8({:?})", e),
                  Self::Message(msg) => write!(f, "Message({:?})", msg),
              }
          }
      }

      pub type ToolResult<T> = std::result::Result<T, ToolError>;
    }
    .into()
}
