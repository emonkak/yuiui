use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::Token;
use syn::parse::{Parse, ParseStream};

#[proc_macro]
pub fn either(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as EitherProcedure);
    let tokens = match parsed {
        EitherProcedure::If(expr) => quote!(#expr),
        EitherProcedure::Match(expr) => quote!(#expr),
    };
    tokens.into()
}

enum EitherProcedure {
    If(syn::ExprIf),
    Match(syn::ExprMatch),
}

impl Parse for EitherProcedure {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![if]) {
            let mut parsed = input.parse()?;
            process_if(&mut parsed, 0)?;
            Ok(EitherProcedure::If(parsed))
        } else if lookahead.peek(Token![match]) {
            let mut parsed = input.parse()?;
            process_match(&mut parsed)?;
            Ok(EitherProcedure::Match(parsed))
        } else {
            Err(lookahead.error())
        }
    }
}

fn process_if(expr: &mut syn::ExprIf, iterations: usize) -> syn::Result<()> {
    if let Some((else_token, else_branch)) = expr.else_branch.as_mut() {
        let then_branch = &expr.then_branch;
        let mut then_branch = quote!(::either::Either::Left(#then_branch));
        for _ in 0..iterations {
            then_branch = quote!(::either::Either::Right(#then_branch));
        }
        then_branch = quote!({ #then_branch });
        expr.then_branch = syn::parse(then_branch.into_token_stream().into())?;

        match else_branch.as_mut() {
            syn::Expr::Block(block) => {
                let mut else_branch = quote!(::either::Either::Right(#block));
                for _ in 0..iterations {
                    else_branch = quote!(::either::Either::Right(#else_branch));
                }
                else_branch = quote!({ #else_branch });
                expr.else_branch = Some((
                    *else_token,
                    syn::parse(else_branch.into_token_stream().into())?)
                );
            }
            syn::Expr::If(next_if) => {
                process_if(next_if, iterations + 1)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn process_match(expr_match: &mut syn::ExprMatch) -> syn::Result<()> {
    let arm_len = expr_match.arms.len();
    if arm_len > 1 {
        let mut iterations = 0;
        for arm in &mut expr_match.arms {
            let body = &arm.body;
            let mut body = if iterations < arm_len - 1 {
                quote!(::either::Either::Left(#body))
            } else {
                quote!(::either::Either::Right(#body))
            };
            for _ in 0..iterations.min(arm_len - 2) {
                body = quote!(::either::Either::Right(#body));
            }
            arm.body = Box::new(syn::parse(body.into_token_stream().into())?);
            iterations += 1;
        }
    }
    Ok(())
}
