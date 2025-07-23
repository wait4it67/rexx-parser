use std::process::ExitCode;
use ungrammar::{Grammar, Rule};

fn main() -> ExitCode {
    println!("cargo::rerun-if-changed=gen/ungrammar.ungrammar");
    let src = include_str!("gen/ungrammar.ungrammar");
    let grammar = src.parse::<Grammar>().unwrap();
    let mut buf = String::new();
    grammar_to_json(&grammar, write_json::object(&mut buf));
    println!("{}", buf);
    ExitCode::SUCCESS
}

fn grammar_to_json(grammar: &Grammar, mut obj: write_json::Object<'_>) {
    for node in grammar.iter() {
        let node = &grammar[node];
        rule_to_json(grammar, &node.rule, obj.object(&node.name));
    }
}

fn rule_to_json(grammar: &Grammar, rule: &Rule, mut obj: write_json::Object) {
    match rule {
        Rule::Labeled { label, rule } => {
            obj.string("label", label);
            rule_to_json(grammar, rule, obj.object("rule"))
        }
        Rule::Node(node) => {
            obj.string("node", &grammar[*node].name);
        }
        Rule::Token(token) => {
            obj.string("token", &grammar[*token].name);
        }
        Rule::Seq(rules) | Rule::Alt(rules) => {
            let tag = match rule {
                Rule::Seq(_) => "seq",
                Rule::Alt(_) => "alt",
                _ => unreachable!(),
            };
            let mut array = obj.array(tag);
            for rule in rules {
                rule_to_json(grammar, rule, array.object());
            }
        }
        Rule::Opt(arg) | Rule::Rep(arg) => {
            let tag = match rule {
                Rule::Opt(_) => "opt",
                Rule::Rep(_) => "rep",
                _ => unreachable!(),
            };
            rule_to_json(grammar, arg, obj.object(tag));
        }
    }
}