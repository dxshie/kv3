#[cfg(feature = "serde")]
#[cfg(test)]
mod tests {
    use log::error;
    use serde::{Deserialize, Serialize};

    use crate::kv3_serde::serde_kv3;

    #[derive(Deserialize, Serialize)]
    struct TestNestedObj {
        obj: NestedObj,
    }

    #[derive(Deserialize, Serialize)]
    struct NestedObj {
        obj1: SomeObj,
    }

    #[test]
    fn kv3_serde_parse_object_nested() {
        let input = r#"
    <!-- test 3 -->
{
  // comment
  obj = {
    obj1 = {}
  }
}
"#;

        match serde_kv3::<TestNestedObj>(input) {
            Ok(_) => {}
            Err(e) => {
                error!("error {:?}", e);
                assert!(true)
            }
        }
    }

    #[derive(Deserialize, Serialize)]
    struct SomeObj;

    #[derive(Deserialize, Serialize)]
    struct ObjTest {
        obj1: SomeObj,
        obj2: SomeObj,
        obj3: SomeObj,
        obj4: SomeObj,
    }

    #[test]
    fn kv3_serde_parse_object() {
        let input = r#"
    <!-- test 3 -->
{
  // comment
  obj1 = {}
  obj2 = { }
  obj3 = { 
  }
  obj4 =
  { 
  }
}
"#;

        match serde_kv3::<ObjTest>(input) {
            Ok(_) => {}
            Err(e) => {
                error!("error {:?}", e);
                assert!(true)
            }
        }
    }

    #[derive(Deserialize, Serialize)]
    struct ArrayTest {
        array1: Vec<i32>,
        array2: Vec<i32>,
        array3: Vec<i32>,
        array4: Vec<i32>,
    }

    #[test]
    fn kv3_serde_parse_array_faulty() {
        let input = r#"
    <!-- test 3 -->
{
  // comment
  array1 = []
  array2 = [ ]
  array3 = [
  ]
  array4 = asd
  [
  ]
}
"#;

        match serde_kv3::<ArrayTest>(input) {
            Ok(_) => {}
            Err(e) => {
                error!("error {:?}", e);
                assert!(true)
            }
        }
    }

    #[test]
    fn kv3_serde_parse_array() {
        let input = r#"
    <!-- test 3 -->
{
  // comment
  array1 = []
  array2 = [ ]
  array3 = [
  ]
  array4 = 
  [
  ]
}
"#;

        match serde_kv3::<ArrayTest>(input) {
            Ok(_) => {}
            Err(e) => {
                error!("error {:?}", e);
                assert!(false)
            }
        }
    }

    #[derive(Deserialize, Serialize)]
    struct Test {
        #[serde(rename = "m_nFlags")]
        flags: f32,
        #[serde(rename = "m_nRefCounter")]
        ref_counter: i32,
    }

    #[test]
    fn kv3_serde_parse_input_numbers() {
        let input = r#"
         <!-- test 3 -->
     {
       m_nFlags = 5.0
       m_nRefCounter = 5
       m_string = "some string"
        multiLineStringValue = """
First line of a multi-line string literal.
Second line of a multi-line string literal.
"""
     }
     "#;

        match serde_kv3::<Test>(input) {
            Ok(_) => {}
            Err(e) => {
                error!("error {:?}", e);
                assert!(false)
            }
        }
    }

    #[derive(Deserialize, Serialize)]
    struct Test2 {
        #[serde(rename = "array1")]
        array1: Vec<i64>,
    }

    #[test]
    fn kv3_serde_parse_input_comments() {
        let input = r#"
         <!-- test 3 -->
     {
         // test 2
         <!-- test 3 -->
         array1 = [1] // test
         num = 5 /*
         ok
         */
         float = 5.0
         obj = {}
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
