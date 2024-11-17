#[cfg(feature = "serde")]
#[cfg(test)]
mod tests {
    use log::error;
    use serde::{Deserialize, Serialize};

    use crate::kv3_serde::serde_kv3;

    #[derive(Deserialize, Serialize)]
    struct TestNestedObj {
        obj: NestedObj,
        data: i32,
    }

    #[derive(Deserialize, Serialize)]
    struct NestedObj {
        obj1: SomeObj,
    }

    #[test]
    fn kv3_serde_parse_object_nested() {
        let input = r#"
{
  data = 5
  obj = {
    obj1 = {
      data = 5
    }
  }
}
"#;

        match serde_kv3::<TestNestedObj>(input) {
            Ok(data) => {
                assert_eq!(data.data, 5);
            }
            Err(e) => {
                error!("error {:?}", e);
                panic!("expected to pass the test")
            }
        }
    }

    #[derive(Deserialize, Serialize)]
    struct SomeObj {
        data: i32,
    }

    #[derive(Deserialize, Serialize)]
    struct SomeObjEmpty {}

    #[derive(Deserialize, Serialize)]
    struct ObjTest {
        obj1: SomeObjEmpty,
        obj2: SomeObjEmpty,
        obj3: SomeObjEmpty,
        obj4: SomeObjEmpty,
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
                panic!("expected to pass the test")
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
                panic!("expected to pass the test")
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
                panic!("expected to pass the test")
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
                panic!("expected to pass the test")
            }
        }
    }
}
