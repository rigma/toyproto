use hickory_resolver::{lookup::Lookup, proto::rr::RData};
use toyproto_syntax::did::{Did, methods::Method};

pub(crate) fn parse_txt_lookup<M>(lookup: Lookup) -> Option<Did<M>>
where
    M: Method,
{
    lookup.answers().iter().find_map(|answer| {
        if let RData::TXT(txt) = &answer.data {
            txt.txt_data
                .iter()
                .filter(|record| record.starts_with(b"did="))
                .map(|record| record[4..].try_into().ok())
                .flatten()
                .next()
        } else {
            None
        }
    })
}
