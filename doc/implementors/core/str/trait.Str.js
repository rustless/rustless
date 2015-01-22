(function() {var implementors = {};
implementors['unicase'] = ["<a class='stability Unstable' title='Unstable: Instead of taking this bound generically, this trait will be replaced with one of slicing syntax, deref coercions, or a more generic conversion trait'></a>impl&lt;S: <a class='trait' href='http://doc.rust-lang.org/nightly/core/ops/trait.Deref.html' title='core::ops::Deref'>Deref</a>&lt;Target=<a href='http://doc.rust-lang.org/nightly/std/primitive.str.html'>str</a>&gt;&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/core/str/trait.Str.html' title='core::str::Str'>Str</a> for <a class='struct' href='unicase/struct.UniCase.html' title='unicase::UniCase'>UniCase</a>&lt;S&gt;",];
implementors['hyper'] = ["<a class='stability Unstable' title='Unstable: waiting on Str stabilization'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/core/str/trait.Str.html' title='core::str::Str'>Str</a> for <a class='struct' href='http://doc.rust-lang.org/nightly/collections/string/struct.String.html' title='collections::string::String'>String</a>","<a class='stability Unstable' title='Unstable: Instead of taking this bound generically, this trait will be replaced with one of slicing syntax, deref coercions, or a more generic conversion trait'></a>impl&lt;'a&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/core/str/trait.Str.html' title='core::str::Str'>Str</a> for <a class='enum' href='http://doc.rust-lang.org/nightly/core/borrow/enum.Cow.html' title='core::borrow::Cow'>Cow</a>&lt;'a, <a class='struct' href='http://doc.rust-lang.org/nightly/collections/string/struct.String.html' title='collections::string::String'>String</a>, <a href='http://doc.rust-lang.org/nightly/std/primitive.str.html'>str</a>&gt;",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
