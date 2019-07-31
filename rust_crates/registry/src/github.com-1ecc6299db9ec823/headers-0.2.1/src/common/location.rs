use ::HeaderValue;

/// `Location` header, defined in
/// [RFC7231](http://tools.ietf.org/html/rfc7231#section-7.1.2)
///
/// The `Location` header field is used in some responses to refer to a
/// specific resource in relation to the response.  The type of
/// relationship is defined by the combination of request method and
/// status code semantics.
///
/// # ABNF
///
/// ```text
/// Location = URI-reference
/// ```
///
/// # Example values
/// * `/People.html#tim`
/// * `http://www.example.net/index.html`
///
/// # Examples
///
//TODO: make this a `Uri`?
#[derive(Clone, Debug, PartialEq, Header)]
pub struct Location(HeaderValue);

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_decode;

    #[test]
    fn absolute_uri() {
        let s = "http://www.example.net/index.html";
        let loc = test_decode::<Location>(&[s]).unwrap();

        assert_eq!(loc, Location(HeaderValue::from_static(s)));
    }

    #[test]
    fn relative_uri_with_fragment() {
        let s = "/People.html#tim";
        let loc = test_decode::<Location>(&[s]).unwrap();

        assert_eq!(loc, Location(HeaderValue::from_static(s)));
    }
}
