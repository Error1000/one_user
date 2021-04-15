extern crate one_user;


#[macro_use]
extern crate lazy_static;

mod test{

use one_user::one_user;
#[one_user]
#[derive(Debug)]
pub struct Test2 {
    item: usize
}


impl Test2{
    fn new(d: usize) -> UnboundTest2{
        UnboundTest2::from(Test2{ item: d })
    }
}

impl test2_binder::OnBind for Test2 {
    fn on_bind<const SLOT: usize>(&mut self) {}
}

}

fn main() {
    let mut b = test::Test2Bouncer::new();
    // let v = test::Test2::default();

   // let mut t1 = Test2::default();
   // let mut t2 = Test2::default();

    //let t1 = t1.bind(&mut b);
   // println!("{:?}", t1);

    //let t2 = t2.bind(&mut b);
   // println!("{:?}", t2);

    /*
    let mut bounce0: BOUNCER = BOUNCER::new().unwrap();
    /*
        println!("Size of BOUNCE<{}>: {} bytes", 0, std::mem::size_of::<BOUNCER<0>>());
        println!("Size of Mutex<[bool; {}]>: {} bytes", 2, std::mem::size_of::<Mutex<[bool; 2]>>());
        println!("Size of multi_bind::Unbound: {} bytes", std::mem::size_of::<multi_test::Unbound>());
        println!("Size of Test: {} bytes", std::mem::size_of::<Test>());
        println!("Size of multi_bind::Bound<{}>: {} bytes", 1, std::mem::size_of::<multi_test::Bound<1>>());
    */

    println!();
    let mut inst = Test::new(2);
    let mut inst2 = Test::new(3);

    let inst2 = inst2.bind(&mut bounce0);
    println!("Hi: {:?}!", *inst2);

    let inst = inst.bind(&mut bounce0);

    println!("Hi: {:?}!", *inst); */
}
