use crate::{
    semantic_services::Semantic,
    utils::{rename::RenameSymbolExtensions, ToCamelCase},
    JsRuleAction,
};
use rome_analyze::{
    context::RuleContext, declare_rule, ActionCategory, Rule, RuleCategory, RuleDiagnostic,
};
use rome_console::markup;
use rome_diagnostics::Applicability;
use rome_js_syntax::{JsFormalParameter, JsIdentifierBinding, JsVariableDeclarator};
use rome_rowan::{AstNode, BatchMutationExt};
use std::{borrow::Cow, iter::once};

declare_rule! {
    /// Enforce camel case naming convention.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// let snake_case;
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// let PascalCase;
    /// ```
    ///
    /// ## Valid
    ///
    /// ```js
    /// let camelCase;
    /// ```
    pub(crate) UseCamelCase {
        version: "0.8.0",
        name: "useCamelCase",
        recommended: false,
    }
}

pub struct State {
    new_name: String,
}

impl Rule for UseCamelCase {
    const CATEGORY: RuleCategory = RuleCategory::Lint;

    type Query = Semantic<JsIdentifierBinding>;
    type State = State;
    type Signals = Option<Self::State>;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let binding = ctx.query();

        let is_variable = binding.parent::<JsVariableDeclarator>().is_some();
        let is_parameter = binding.parent::<JsFormalParameter>().is_some();
        if is_variable || is_parameter {
            let name = binding.name_token().ok()?;
            let name = name.text_trimmed();

            if name.starts_with('_') {
                return None;
            }

            match name.to_camel_case() {
                Cow::Borrowed(_) => None,
                Cow::Owned(new_name) => Some(State { new_name }),
            }
        } else {
            None
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, _: &Self::State) -> Option<RuleDiagnostic> {
        let binding = ctx.query();

        let diag = RuleDiagnostic::warning(
            binding.syntax().text_trimmed_range(),
            markup! {
                "Prefer symbols names in camel case."
            },
        );

        Some(diag)
    }

    fn action(ctx: &RuleContext<Self>, State { new_name }: &Self::State) -> Option<JsRuleAction> {
        let model = ctx.model();
        let mut batch = ctx.root().begin();

        let candidates = (2..).map(|i| format!("{}{}", new_name, i).into());
        let candidates = once(Cow::from(new_name)).chain(candidates);
        if batch.rename_node_declaration_with_retry(model, ctx.query().clone(), candidates) {
            Some(JsRuleAction {
                category: ActionCategory::Refactor,
                applicability: Applicability::Always,
                message: markup! { "Rename this symbol to camel case" }.to_owned(),
                mutation: batch,
            })
        } else {
            None
        }
    }
}
