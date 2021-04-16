extern crate one_user;
#[macro_use]
extern crate lazy_static;

mod test{

    use one_user::one_user;
    #[one_user]
    #[derive(Debug)]
    pub struct Test<T>
    where T: Debug{
        item: T
    }


    impl Test{
        pub fn new(d: usize) -> UnboundTest{
            UnboundTest::from(Test{ item: d })
        }
    }

    impl test_binder::OnBind for Test {
        fn on_bind<const SLOT: usize>(&mut self) {}
    }

}

fn main() {
    let mut b = test::TestBouncer::new();
    // let mut b2 = test::TestBouncer::new(); // Error: Can't create more than 1 bouncer if no arguments are specified to the macro

    let mut t1 = test::Test::new(1);
    let mut t2 = test::Test::new(2);
    // Here we cannot use t1 or t2 because they are not bound
    //println!("{:?}", t1); // Error

    // This works:
    
    let bound_t1 = t1.bind(&mut b);
    println!("{:?}", *bound_t1); // Now we can use t1
    
    let bound_t2 = t2.bind(&mut b);
    println!("{:?}", *bound_t2); // Now we can use t2


    // This dosen't:
/*
    let bound_t1 = t1.bind(&mut b);
    let bound_t2 = t2.bind(&mut b); // Error: Can't have more than one user at a time, a.k.a you can only have access to one instance of the Test struct at a time


    println!("{:?}", *bound_t1); // Now we can use t1
    
    println!("{:?}", *bound_t2); // Now we can use t2
*/

}
