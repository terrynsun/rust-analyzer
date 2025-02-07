use super::*;

pub(super) fn opt_generic_param_list(p: &mut Parser) {
    if p.at(T![<]) {
        generic_param_list(p);
    }
}

// test generic_param_list
// fn f<T: Clone>() {}
fn generic_param_list(p: &mut Parser) {
    assert!(p.at(T![<]));
    let m = p.start();
    p.bump(T![<]);

    while !p.at(EOF) && !p.at(T![>]) {
        generic_param(p);
        if !p.at(T![>]) && !p.expect(T![,]) {
            break;
        }
    }
    p.expect(T![>]);
    m.complete(p, GENERIC_PARAM_LIST);
}

fn generic_param(p: &mut Parser) {
    let m = p.start();
    // test generic_param_attribute
    // fn foo<#[lt_attr] 'a, #[t_attr] T>() {}
    attributes::outer_attrs(p);
    match p.current() {
        LIFETIME_IDENT => lifetime_param(p, m),
        IDENT => type_param(p, m),
        T![const] => const_param(p, m),
        _ => {
            m.abandon(p);
            p.err_and_bump("expected type parameter")
        }
    }
}

// test lifetime_param
// fn f<'a: 'b>() {}
fn lifetime_param(p: &mut Parser, m: Marker) {
    assert!(p.at(LIFETIME_IDENT));
    lifetime(p);
    if p.at(T![:]) {
        lifetime_bounds(p);
    }
    m.complete(p, LIFETIME_PARAM);
}

// test type_param
// fn f<T: Clone>() {}
fn type_param(p: &mut Parser, m: Marker) {
    assert!(p.at(IDENT));
    name(p);
    if p.at(T![:]) {
        bounds(p);
    }
    if p.at(T![=]) {
        // test type_param_default
        // struct S<T = i32>;
        p.bump(T![=]);
        types::type_(p)
    }
    m.complete(p, TYPE_PARAM);
}

// test const_param
// struct S<const N: u32>;
fn const_param(p: &mut Parser, m: Marker) {
    p.bump(T![const]);
    name(p);
    if p.at(T![:]) {
        types::ascription(p);
    } else {
        p.error("missing type for const parameter");
    }

    if p.at(T![=]) {
        // test const_param_defaults
        // struct A<const N: i32 = -1>;
        p.bump(T![=]);
        generic_args::const_arg(p);
    }

    m.complete(p, CONST_PARAM);
}

fn lifetime_bounds(p: &mut Parser) {
    assert!(p.at(T![:]));
    p.bump(T![:]);
    while p.at(LIFETIME_IDENT) {
        lifetime(p);
        if !p.eat(T![+]) {
            break;
        }
    }
}

// test type_param_bounds
// struct S<T: 'a + ?Sized + (Copy)>;
pub(super) fn bounds(p: &mut Parser) {
    assert!(p.at(T![:]));
    p.bump(T![:]);
    bounds_without_colon(p);
}

pub(super) fn bounds_without_colon(p: &mut Parser) {
    let m = p.start();
    bounds_without_colon_m(p, m);
}

pub(super) fn bounds_without_colon_m(p: &mut Parser, marker: Marker) -> CompletedMarker {
    while type_bound(p) {
        if !p.eat(T![+]) {
            break;
        }
    }
    marker.complete(p, TYPE_BOUND_LIST)
}

fn type_bound(p: &mut Parser) -> bool {
    let m = p.start();
    let has_paren = p.eat(T!['(']);
    p.eat(T![?]);
    match p.current() {
        LIFETIME_IDENT => lifetime(p),
        T![for] => types::for_type(p, false),
        _ if paths::is_use_path_start(p) => types::path_type_(p, false),
        _ => {
            m.abandon(p);
            return false;
        }
    }
    if has_paren {
        p.expect(T![')']);
    }
    m.complete(p, TYPE_BOUND);

    true
}

// test where_clause
// fn foo()
// where
//    'a: 'b + 'c,
//    T: Clone + Copy + 'static,
//    Iterator::Item: 'a,
//    <T as Iterator>::Item: 'a
// {}
pub(super) fn opt_where_clause(p: &mut Parser) {
    if !p.at(T![where]) {
        return;
    }
    let m = p.start();
    p.bump(T![where]);

    while is_where_predicate(p) {
        where_predicate(p);

        let comma = p.eat(T![,]);

        match p.current() {
            T!['{'] | T![;] | T![=] => break,
            _ => (),
        }

        if !comma {
            p.error("expected comma");
        }
    }

    m.complete(p, WHERE_CLAUSE);

    fn is_where_predicate(p: &mut Parser) -> bool {
        match p.current() {
            LIFETIME_IDENT => true,
            T![impl] => false,
            token => types::TYPE_FIRST.contains(token),
        }
    }
}

fn where_predicate(p: &mut Parser) {
    let m = p.start();
    match p.current() {
        LIFETIME_IDENT => {
            lifetime(p);
            if p.at(T![:]) {
                bounds(p);
            } else {
                p.error("expected colon");
            }
        }
        T![impl] => {
            p.error("expected lifetime or type");
        }
        _ => {
            if p.at(T![for]) {
                // test where_pred_for
                // fn for_trait<F>()
                // where
                //    for<'a> F: Fn(&'a str)
                // { }
                types::for_binder(p);
            }

            types::type_(p);

            if p.at(T![:]) {
                bounds(p);
            } else {
                p.error("expected colon");
            }
        }
    }
    m.complete(p, WHERE_PRED);
}
