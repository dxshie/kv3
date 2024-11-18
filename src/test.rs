#[cfg(feature = "serde")]
#[cfg(test)]
mod tests {
    use log::{error, info};
    use serde::{Deserialize, Serialize};

    use crate::{kv3_serde::serde_kv3, parse_kv3};

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

    #[derive(Debug, Deserialize, Serialize)]
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

    #[derive(Debug, Serialize, Deserialize)]
    struct ArrayTest2 {
        array_ints: Vec<i64>,
    }

    #[test]
    fn kv3_serde_parse_array() {
        let input = r#"
     {
       array_empty = []
       array_whitespace = [ ]
       array_mixed = [1, 2.5, true, "string"]
       array_nested = [1, [2, 3], 4]
       array_floats = [1.1, -2.2, 3.3]
       array_ints = [1, 2, 3, 4]
       array_nl = 
       [
       ]
       m_MassProperties = 
       [
           11629.881836,
           -4467.779785,
           -2793.832764,
           -703.485962
       ]
       m_MassProperties2 = 
       [
           5.0,
           4.0,
       ]
     }
    "#;

        match serde_kv3::<ArrayTest2>(input) {
            Ok(data) => {
                info!("{:?}", data);
            }
            Err(e) => {
                error!("error {:?}", e);
                panic!("expected to pass the test {:?}", e)
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

    #[derive(Debug, Deserialize, Serialize)]
    struct WorldPhys {
        #[serde(rename = "m_nFlags")]
        flags: i64,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct SerdeParseTest2 {
        #[serde(rename = "num")]
        num: i64,
    }
    #[test]
    fn kv3_serde_parse_test2() {
        let input = r#"
<!-- kv3 encoding:text:version{e21c7f3c-8a33-41c5-9977-a76d3a32aa0d} format:generic:version{7412167c-06e9-4698-aff2-e63eb59037e7} -->
          {
            m_nFlags = 0
            m_nRefCounter = 0
            m_bonesHash = 
            [
            ]
            m_boneNames = 
            [
            ]
            m_indexNames = 
            [
            ]
            m_indexHash = 
            [
            ]
            m_Planes = 
            #[
                00 00 00 00 00 00 00 00 00 00 80 3F 00 00 40 43 E8 83 84 3E EB 46 77 BF C2 BD 77 35 42 C9 EB C3
                40 48 24 3F C9 13 30 3E 24 56 3F BF 49 AD 05 C4 00 00 00 00 00 00 00 00 00 00 80 BF 00 00 38 C3
                E8 83 84 BE EB 46 77 3F C2 BD 77 B5 42 C9 EF 43 E9 46 77 BF F4 83 84 BE 00 00 00 00 CD 35 18 44
                E9 46 77 3F F4 83 84 3E 00 00 00 00 CD 35 13 C4 F5 83 84 3E EB 46 77 BF 00 00 00 00 52 C9 EF C3
                E9 46 77 BF F4 83 84 BE 00 00 00 00 CD 35 18 44 8B 3D 80 34 B8 4C 6F B5 00 00 80 BF 1D 00 40 C3
                E9 46 77 BF F4 83 84 BE 00 00 00 00 CD 35 18 44 F5 83 84 BE EB 46 77 3F 00 00 00 00 52 C9 EB 43
                E9 46 77 3F F4 83 84 3E 00 00 00 00 CD 35 13 C4 8B 3D 80 34 B8 4C 6F B5 00 00 80 BF 1D 00 40 C3
                ED 46 77 BF D1 83 84 BE 00 00 00 00 D4 35 18 44 69 3D 80 B4 BB 4C 6F 35 00 00 80 3F 1C 00 38 43
                ED 46 77 3F D1 83 84 3E 00 00 00 00 D4 75 15 C4 D2 83 84 BE EF 46 77 3F 00 00 00 00 3C C9 EB 43
                F4 83 84 3E E9 46 77 BF 00 00 00 00 50 C9 EB C3 EB 46 77 BF F5 83 84 BE 00 00 00 00 CE 35 13 44
                F4 83 84 BE E9 46 77 3F 00 00 00 00 50 C9 EF 43 1C D1 38 BF 14 16 46 BE D6 13 2A BF E8 45 9C 43
                19 D1 38 BF 54 16 46 BE D6 13 2A BF 2C 4B A2 43 0C 84 84 3E E7 46 77 BF 00 00 2E 35 5C C9 EF C3
                19 D1 38 3F 54 16 46 3E D6 13 2A 3F DA 45 9C C3 44 48 24 BF 8C 13 30 BE 25 56 3F 3F 50 AD 05 44
                19 D1 38 3F 54 16 46 3E D6 13 2A 3F DA 45 9C C3 0C 84 84 BE E7 46 77 3F 00 00 2E B5 5B C9 EB 43
                19 D1 38 BF 54 16 46 BE D6 13 2A BF 2C 4B A2 43 44 48 24 BF 8C 13 30 BE 25 56 3F 3F 50 AD 05 44
                F4 83 84 BE E9 46 77 3F 00 00 00 00 50 C9 EF 43 EB 46 77 BF F5 83 84 BE 00 00 00 80 CD 75 15 44
                F4 83 84 3E E9 46 77 BF 00 00 00 00 50 C9 EB C3 1C D1 38 3F 14 16 46 3E D6 13 2A 3F 3A 4B A2 C3
                F4 83 84 3E E9 46 77 BF 00 00 00 00 50 C9 EB C3 EB 46 77 3F F5 83 84 3E 00 00 00 00 CE 35 18 C4
                F4 83 84 BE E9 46 77 3F 00 00 00 00 50 C9 EF 43 00 00 00 00 00 00 00 00 00 00 80 3F 00 00 38 43
                ED 46 77 3F D1 83 84 3E 00 00 00 00 D4 75 15 C4 69 3D 80 B4 BB 4C 6F 35 00 00 80 3F 1D 00 38 43
                ED 46 77 BF D1 83 84 BE 00 00 00 00 D4 35 18 44 D2 83 84 3E EF 46 77 BF 00 00 00 00 3C C9 EF C3
                F4 83 84 BE E9 46 77 3F 00 00 00 00 50 C9 EF 43 EB 46 77 3F F5 83 84 3E 00 00 00 80 CE 35 18 C4
                F4 83 84 3E E9 46 77 BF 00 00 00 00 50 C9 EB C3 00 00 00 00 00 00 00 80 00 00 80 BF 00 00 40 C3
                00 00 00 00 00 00 00 00 00 00 80 BF 00 00 38 C3 F5 83 84 BE EB 46 77 3F 00 00 00 00 52 C9 EB 43
                00 00 00 00 00 00 00 00 00 00 80 3F 00 00 40 43 EB 46 77 3F E8 83 84 3E 00 00 00 80 D0 35 18 C4
                00 00 00 00 00 00 00 00 00 00 80 3F 00 00 40 43 F5 83 84 3E EB 46 77 BF 00 00 00 00 52 C9 EF C3
                00 00 00 00 00 00 00 00 00 00 80 BF 00 00 38 C3 EB 46 77 3F E8 83 84 3E 00 00 00 00 D0 35 18 C4
            ]
            m_bindPose = 
            [
            ]
              // test 2
              <!-- test 3 -->
              array1 = [1] // test
              num = 5 /*
              ok
              */
              float = 5.0
              obj = {}
              obj2 = 
              {
                arr = [5]
                arr2 = #[FF FF FF]
              }
            m_MassProperties = 
            [
                11629.881836,
                -4467.779785,
                -2793.832764,
                -703.485962,
                -4467.779785,
                27106.720703,
                -748.609741,
                303.850525,
                -2793.832764,
                -748.609741,
                28452.515625,
                188.387100,
            ]
            m_parts = 
            [
                
                {
                    m_nFlags = 2
                    m_flMass = 0.000000
                    m_rnShape = 
                    {
                        m_spheres = 
                        [
                        ]
                        m_capsules = 
                        [
                        ]
                        m_hulls = 
                        [
                        ]
                    }
                }
            ]
            test = 
            [
              5,
            ]
        }
          "#;
        match serde_kv3::<SerdeParseTest2>(input) {
            Ok(data) => {
                println!("{:?}", data);
            }
            Err(e) => {
                error!("error {:?}", e);
                panic!("expected to pass the test {:?}", e);
            }
        }
    }
}
