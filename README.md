# One User

    This crate provides a way to make sure a struct is only used by "N users" at a time.
    It achives this by providing a procedural macro which generates code for forcing only N active "views" (a.k.a mut references to some instances of the struct) at a time.
    Under the hood this is done by having a bound and unbound version of your struct and then requiring a &mut to a bouncer to give you a bound to a slot,
    and since bouncers are unique per slot there can only ever be N bounds whre N is the numbers of slots. It also provides a hook for reacting to an instance of your struct being bound to a slot.

