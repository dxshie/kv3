#[cfg(test)]
mod tests {
    use log::error;
    use serde::{Deserialize, Serialize};

    use crate::kv3_serde::serde_kv3;

    #[derive(Deserialize, Serialize)]
    struct Test {
        #[serde(rename = "m_nFlags")]
        flags: f32,
        #[serde(rename = "m_nRefCounter")]
        ref_counter: i32,
    }

    #[derive(Deserialize, Serialize)]
    struct Test2 {
        #[serde(rename = "array1")]
        array1: Vec<i64>,
    }

    #[test]
    fn kv3_serde_parse_input2_array_object() {
        env_logger::init();
        let input = r#"
{
    array1 = 1
}
"#;

        match serde_kv3::<Test2>(input) {
            Ok(_) => {}
            Err(e) => {
                error!("error {:?}", e);
                assert!(false)
            }
        }
    }
}
