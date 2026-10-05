use std::{fmt, marker::PhantomData};
use toyproto_syntax::did::{
    Did,
    methods::{Method, Universal},
};

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document<M: Method = Universal> {
    pub id: Did<M>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub also_known_as: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verification_method: Vec<VerificationMethod>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub service: Vec<Service>,
}

impl<'de, M> serde::Deserialize<'de> for Document<M>
where
    M: Method,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, IgnoredAny, MapAccess, Visitor};

        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Fields {
            Id,
            AlsoKnownAs,
            VerificationMethod,
            Service,
            #[allow(dead_code)]
            Unknown(String),
        }

        struct DocumentVisitor<M: Method>(PhantomData<M>);
        impl<'de, M> Visitor<'de> for DocumentVisitor<M>
        where
            M: Method,
        {
            type Value = Document<M>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "expecting a DID document")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut id = None;
                let mut also_known_as = None;
                let mut verification_method = None;
                let mut service = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("did"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Fields::AlsoKnownAs => {
                            if also_known_as.is_some() {
                                return Err(de::Error::duplicate_field("also_known_as"));
                            }
                            also_known_as = Some(map.next_value()?);
                        }
                        Fields::VerificationMethod => {
                            if verification_method.is_some() {
                                return Err(de::Error::duplicate_field("verification_method"));
                            }
                            verification_method = Some(map.next_value()?);
                        }
                        Fields::Service => {
                            if service.is_some() {
                                return Err(de::Error::duplicate_field("service"));
                            }
                            service = Some(map.next_value()?);
                        }
                        Fields::Unknown(_) => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let also_known_as = also_known_as.unwrap_or_default();
                let verification_method = verification_method.unwrap_or_default();
                let service = service.unwrap_or_default();

                Ok(Self::Value {
                    id,
                    also_known_as,
                    verification_method,
                    service,
                })
            }
        }

        deserializer.deserialize_struct("Document", &[], DocumentVisitor(PhantomData))
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Service {
    pub id: String,

    #[serde(rename = "type")]
    pub ty: String,

    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct VerificationMethod {
    pub id: String,

    #[serde(rename = "type")]
    pub ty: String,

    pub controller: String,

    #[serde(rename = "publicKeyMultibase")]
    pub public_key_multibase: String,
}
