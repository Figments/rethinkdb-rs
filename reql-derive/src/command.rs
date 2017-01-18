use std::collections::HashMap;

use quote::{Tokens, ToTokens};
use syn::{Ident, Lit, MetaItem, NestedMetaItem, MacroInput};
use case::CaseExt;

pub struct Command {
    ast: MacroInput,
}

impl Command {
    pub fn new(ast: MacroInput) -> Command {
        Command {
            ast: ast,
        }
    }

    pub fn derive(&self) -> Tokens {
        let title = self.title();
        let typ = self.typ();
        let docs = self.docs();
        let sig = self.sig();
        let body = self.body();

        quote! {
            #title
            pub trait #typ {
                #docs
                fn #sig;
            }

            impl #typ for ::Command {
                fn #sig {
                    #body
                }
            }
        }
    }

    fn command(&self) -> &MetaItem {
        let mut commands = self.ast.attrs.iter().filter(|attr| {
                if let MetaItem::List(ref attr, _) = attr.value {
                    attr == "command"
                } else {
                    false
                }
            })
        .peekable();
        let command = commands.peek().expect("command attribute not defined");
        let mut meta_items = Vec::new();
        match command.value {
            MetaItem::List(_, ref items) => {
                if items.len() != 1 {
                    panic!("A command attribute must only carry one command.");
                }
                for item in items {
                    match *item {
                        NestedMetaItem::MetaItem(ref command) => { meta_items.push(command); },
                        NestedMetaItem::Literal(_) => { panic!("A command attribute should not define commands as literals"); },
                    }
                }
            }
            _ => { panic!("A command attribute must be defined as a list of items. That is `#[command(..)]`."); },
        };
        meta_items.iter().next().unwrap()
    }

    fn name(&self) -> &Ident {
        match *self.command() {
            MetaItem::Word(ref name) => name,
            MetaItem::List(ref name, _) => name,
            MetaItem::NameValue(..) => { panic!("A command must not be a name value pair"); },
        }
    }

    fn value(&self) -> Vec<&NestedMetaItem> {
        let mut value = Vec::new();
        match *self.command() {
            MetaItem::Word(_) => { },
            MetaItem::List(_, ref items) => {
                for item in items {
                    value.push(item);
                }
            }
            MetaItem::NameValue(..) => { panic!("A command must not be a name value pair"); },
        }
        value
    }

    fn args(&self) -> Vec<(Ident, Ident)> {
        let mut args = Vec::new();
        for item in self.value() {
            if let NestedMetaItem::MetaItem(MetaItem::List(ref name, ref value)) = *item {
                if name == "args" {
                    for item in value {
                        match *item {
                            NestedMetaItem::MetaItem(ref item) => {
                                if let MetaItem::NameValue(ref name, ref typ) = *item {
                                    match *typ {
                                        Lit::Str(ref typ, _) => {
                                            args.push((name.clone(), Ident::new(typ.as_str())));
                                        }
                                        _ => { panic!("An arg name must be defined as a string"); },
                                    };
                                } else {
                                    panic!("Args must be key value pairs");
                                }
                            },
                            NestedMetaItem::Literal(_) => { panic!("Args must not be literals"); },
                        }
                    }
                }
            }
        }
        args
    }

    fn title(&self) -> Tokens {
        let docs = self.docs();
        for line in docs.as_str().lines() {
            let doc = line.trim_left_matches("///").trim();
            if !doc.is_empty() {
                return quote!(#[doc=#doc]);
            }
        }
        Tokens::new()
    }

    fn typ(&self) -> Tokens {
        let name = self.name().to_string().to_camel();
        let typ = Ident::new(name);
        quote!(#typ)
    }

    fn docs(&self) -> Tokens {
        let mut docs = Tokens::new();
        for attr in self.ast.attrs.iter() {
            if attr.is_sugared_doc {
                attr.to_tokens(&mut docs);
            }
        }
        docs
    }

    fn sig(&self) -> Tokens {
        let mut types = HashMap::new();

        let func = self.name().clone();
        let mut generics = Tokens::new();
        let mut func_args = Tokens::new();
        let mut _where = Tokens::new();

        let args = self.args();
        if !args.is_empty() {
            let mut gen = Vec::new();
            let mut whe = Vec::new();

            gen.push(quote!(<));
            whe.push(quote!(where));

            for (arg, typ) in args {
                let typ_str = typ.to_string();
                if let None = types.get(&typ_str) {
                    gen.push(quote!(#typ));
                    gen.push(quote!(,));
                    whe.push(quote!(#typ: ::IntoArg));
                    whe.push(quote!(,));
                    types.insert(typ_str, ());
                }
                func_args.append_all(&[ quote!(, #arg: #typ) ]);
            }

            gen.pop();
            whe.pop();

            gen.push(quote!(>));

            generics.append_all(gen);
            _where.append_all(whe);
        }

        quote! {
            #func #generics (self #func_args) -> ::Command #_where
        }
    }

    fn body(&self) -> Tokens {
        let cmd_type = Ident::new(self.name().to_string().to_snake().to_uppercase());
        let mut args = Tokens::new();
        for (arg, _) in self.args() {
            let token = quote! {
                for arg in #arg.into_arg() {
                    term.mut_args().push(arg);
                }
            };
            token.to_tokens(&mut args);
        }

        quote! {
            let mut term = ::ql2::proto::Term::new();
            term.set_field_type(::ql2::proto::Term_TermType::#cmd_type);
            if let Some(cmd) = self.term {
                let args = ::protobuf::repeated::RepeatedField::from_vec(vec![cmd]);
                term.set_args(args);
            }
            #args
            ::Command {
                term: Some(term),
                idx: self.idx + 1,
            }
        }
    }
}
