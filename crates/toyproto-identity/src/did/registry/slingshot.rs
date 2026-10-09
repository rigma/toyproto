use crate::did::document::{Document, Service, VerificationMethod};
use serde::de::{self, Deserialize, Deserializer, IgnoredAny, MapAccess, Visitor};
use std::marker::PhantomData;
use toyproto_syntax::did::{
    Did,
    methods::{Method, Universal},
};

#[derive(Clone, Debug)]
pub(super) struct MiniDoc<M: Method = Universal> {
    did: Did<M>,
    handle: String,
    pds: String,
    signing_key: String,
}

impl<'de, M> Deserialize<'de> for MiniDoc<M>
where
    M: Method,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Fields {
            Did,
            Handle,
            Pds,
            SigningKey,
            #[allow(dead_code)]
            Unknown(String),
        }

        struct MiniDocVisitor<M: Method>(PhantomData<M>);

        impl<'de, M> Visitor<'de> for MiniDocVisitor<M>
        where
            M: Method,
        {
            type Value = MiniDoc<M>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "expecting a Slingshot minidoc")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut did = None;
                let mut handle = None;
                let mut pds = None;
                let mut signing_key = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::Did => {
                            if did.is_some() {
                                return Err(de::Error::duplicate_field("did"));
                            }
                            did = Some(map.next_value()?);
                        }
                        Fields::Handle => {
                            if handle.is_some() {
                                return Err(de::Error::duplicate_field("handle"));
                            }
                            handle = Some(map.next_value()?);
                        }
                        Fields::Pds => {
                            if pds.is_some() {
                                return Err(de::Error::duplicate_field("pds"));
                            }
                            pds = Some(map.next_value()?);
                        }
                        Fields::SigningKey => {
                            if signing_key.is_some() {
                                return Err(de::Error::duplicate_field("signing_key"));
                            }
                            signing_key = Some(map.next_value()?);
                        }
                        Fields::Unknown(_) => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let did = did.ok_or_else(|| de::Error::missing_field("did"))?;
                let handle = handle.ok_or_else(|| de::Error::missing_field("handle"))?;
                let pds = pds.ok_or_else(|| de::Error::missing_field("pds"))?;
                let signing_key =
                    signing_key.ok_or_else(|| de::Error::missing_field("signing_key"))?;

                Ok(Self::Value {
                    did,
                    handle,
                    pds,
                    signing_key,
                })
            }
        }

        deserializer.deserialize_struct("MiniDoc", &[], MiniDocVisitor(PhantomData))
    }
}

impl<M> From<MiniDoc<M>> for Document<M>
where
    M: Method,
{
    fn from(value: MiniDoc<M>) -> Self {
        Self {
            id: value.did,
            also_known_as: vec![format!("at://{}", value.handle)],
            verification_method: vec![VerificationMethod {
                public_key_multibase: value.signing_key,
                ..Default::default()
            }],
            service: vec![Service {
                service_endpoint: value.pds,
                ..Default::default()
            }],
        }
    }
}
