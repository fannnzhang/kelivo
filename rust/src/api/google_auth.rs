use anyhow::{anyhow, Context, Result as AnyResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

#[flutter_rust_bridge::frb]
pub fn create_google_auth_jwt(
    client_email: String,
    private_key_pem: String,
    token_uri: String,
    scopes: Vec<String>,
) -> Result<String, String> {
    create_google_auth_jwt_impl(client_email, private_key_pem, token_uri, scopes)
        .map_err(|err| err.to_string())
}

fn create_google_auth_jwt_impl(
    client_email: String,
    private_key_pem: String,
    token_uri: String,
    scopes: Vec<String>,
) -> AnyResult<String> {
    let email = client_email.trim();
    if email.is_empty() {
        return Err(anyhow!("client_email is required"));
    }

    let audience = token_uri.trim();
    if audience.is_empty() {
        return Err(anyhow!("token_uri is required"));
    }

    let scope = normalize_scopes(&scopes)?;

    let signing_key = EncodingKey::from_rsa_pem(private_key_pem.trim().as_bytes())
        .context("invalid RSA private key")?;

    let now = Utc::now();
    let issued_at = now.timestamp();
    let expires_at = (now + Duration::hours(1)).timestamp();

    let claims = Claims {
        iss: email.to_owned(),
        scope,
        aud: audience.to_owned(),
        iat: issued_at,
        exp: expires_at,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.typ = Some("JWT".to_string());

    encode(&header, &claims, &signing_key).context("failed to encode JWT")
}

fn normalize_scopes(scopes: &[String]) -> AnyResult<String> {
    let values: Vec<String> = scopes
        .iter()
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .map(|scope| scope.to_string())
        .collect();

    if values.is_empty() {
        return Err(anyhow!("at least one scope is required"));
    }

    Ok(values.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    fn sample_scopes() -> Vec<String> {
        vec!["https://www.googleapis.com/auth/cloud-platform".to_string()]
    }

    fn sample_token_uri() -> String {
        "https://oauth2.googleapis.com/token".to_string()
    }

    const SAMPLE_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQDENebwQEsOuibN\nzaiNwCoH50H5AN0Xkk8DXmV+qrSeQxI9BcDMulKMG85tSYdcAPklB4tK3LdRep2b\ngiBQLZpdcxbnVaux/bSDgEJFMljG8SvdI58JnqnZczmCR4z8c5JtwV2a1xJkEI25\n4MgKfnyauBFIcECdf/cHOLQ3vt3hp/3zrd2/wMGpVkzbtMZymw7V5qolaTeUMDi+\nt/wstd/1GthrTZjPgC56JHcPuwAdDCoOoVvFH2rTuTHCXLZ/ysR7/bVdTnHq844s\nJ3Y808YI/lLEAzNeF9GGIMit5kEIQhqfJDDNNO6LuOerDfCp+dR29nhfXgV13BoM\nUOqYuZNrAgMBAAECggEBAIe0PCBoZ0EtUI9AsVYw1SEYOhHNHh0ibRGIZSwhRsC6\n5M9dvkYai+MpjDEcDMl+RtLsj9NcKlHpOz6F1nF9yOjfI7UmFJULQqE8wRj4xFv8\nC3lsHJ/bo9+oiNrpP8KO8HSGq4XfQHIC3PL1W4Wei/G6YwG12YrVKXZOKp7vnKhn\ngNsGIViHgdnvSn4rsFPTXNGSVXJKyUtpgLIYDknwM/TjLNZMH2SyWVOFxAzu6roL\njyS9ejkhdIsJ3BzwQPAi1eYO1jx+radA8LAjLFFiAUbdUHAawX4z8aw8OTpwlV9O\n69kUP9JoHU/AudL+u2g0IRCMPqOwUxqATRbcBT3yI4kCgYEA5ymvUTZwZyReDUHU\ntkjpjtPCf5wNjozB80lLule3yl66kzW7YIPlrySjsoIZvqUjiW93Q6GUjZ1BEuiq\nvUtFo2OQWHKydMvx2WDW6Jmkz+NHAJqAGwDuc9lx0qbvpmJrPyNp0ivtvAQc040j\niajZlwXM13Cg0po3LiJR9CX4As8CgYEA2UrT1Qudv+Jt8+xGvPp8GJL09hqi5h7k\nnxHY2hBqmNZU3+JbI2Bwm3A71pU3E/9uztSrK6BJzA62NNmRonO3D8vyDrbnXTVV\nvDh+dkk9WTeLAwoB3nlEuYwM/r5lKWorCaM0tsFbtkLI6G6tPVLaWKwRYxPlfdhv\nGB5snugk/KUCgYEA1uU0mz4NnoT7fj2Nvmvn3CRWMwViwPtvrnicEr5LGLGZpxKT\nf/T+CCT6nQ8/WbDxaWmbKN9EV6YyAZ8UYudf9LWxUdhGuDeEPL4+63sx8STrM89t\nei0Sf9ZMbzVLopTp+Ic2b/DwkBIOrkgOGoJCzZfRxxJoH4DH/XZgU6Uce5cCgYB/\nqnglPwrzF200mEjDdYP9yDIGeoXlIJeAYjL/hA+tNphtZgfYFCx1Fw8BN3BB3pzT\nBp5/JlDVhHtHN+FaChSvQks4m/v5hoGV8bdTdhqGVZzmLzYD6UoYnhFwhprXJ1qb\n8fjhu19QCZMTBRvh4NEKsiYRCTqXotc6231EK+63RQKBgQDSKmxKcyhfXMpwXQL4\n3+VR+SXW2+1j/JbhkSLbSGRPFYeIpZbOvMPXyMMlI7dUJfS0kwyXKx7NtlDi50Oa\naBsbexf6UCMBdp+ArC60cRcvDxFFMn3q8rSF+kURg7fVfLmOBcJEZgbm7WXk0JAb\nCh41joyvUckg5bqMFtQX1p0Xng==\n-----END PRIVATE KEY-----\n";

    #[test]
    fn creates_signed_jwt() {
        let token = create_google_auth_jwt_impl(
            "test-service@example.iam.gserviceaccount.com".to_string(),
            SAMPLE_PRIVATE_KEY.to_string(),
            sample_token_uri(),
            sample_scopes(),
        )
        .expect("expected token");

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;
        validation.validate_exp = false;
        validation.insecure_disable_signature_validation();
        let decoded = decode::<Claims>(&token, &DecodingKey::from_secret(&[]), &validation)
            .expect("decode token");
        assert_eq!(
            decoded.claims.iss,
            "test-service@example.iam.gserviceaccount.com"
        );
        assert_eq!(
            decoded.claims.scope,
            "https://www.googleapis.com/auth/cloud-platform"
        );
        assert_eq!(decoded.claims.aud, sample_token_uri());
        assert_eq!(decoded.claims.exp - decoded.claims.iat, 3600);
    }

    #[test]
    fn returns_error_for_invalid_key() {
        let err = create_google_auth_jwt_impl(
            "user@example.com".to_string(),
            "INVALID_KEY".to_string(),
            sample_token_uri(),
            sample_scopes(),
        )
        .expect_err("expected failure");

        assert!(err.to_string().contains("invalid RSA private key"));
    }

    #[test]
    fn requires_non_empty_scopes() {
        let err = create_google_auth_jwt_impl(
            "user@example.com".to_string(),
            SAMPLE_PRIVATE_KEY.to_string(),
            sample_token_uri(),
            vec!["".into()],
        )
        .expect_err("expected failure");

        assert!(err.to_string().contains("at least one scope"));
    }
}
