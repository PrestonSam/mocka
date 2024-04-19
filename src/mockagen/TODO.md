# Mockagen

The sequencer works but is messy.
Some functions take RuleData, others Pair<'_, Rule>, others AnnotatedPairs<'_>. It's confusing.
See if you can convert Pairs<> into a simpler type like Vec<Pair<'_, Rule>> and see if you can turn
Pair<'_, Rule> into RuleData or something like it.
I want everything to have the same type signature so that the code is a little more homogenous & consistent

Alright, so I don't forget, the reason I created RuleData was to let me destructure the data using match expressions. Pair & Pairs are opaque structs and that makes them more difficult to work with.

I think I should make a struct that has the get_rules_arr_from_pairs stuff built right in.

The purpose of the Providence struct is to create annotated errors.
Hmm should I create a duplicate of the Providence struct that represents an owned copy?





What sort of type signature should I create for the evaluator?

In the last implementation, I returned a hash of functions with their names as the key.
Of course the functions that have a USING clause must accept arguments, too.
Presumably that means I'd have to order the rules by their dependencies?

In my last method, I generated and returned lambdas. That's a very functional approach.
What would it be like if I used structs this time to describe what functions to call?
Would that be simpler? Would it be faster?
I'm not confident it would be, tbh.
Although it'd definitely be easier to serialise.



I think I need to explcitly separate the match values from the assignment values.
I also need to separate out the primitive values from the higher order values.
This should make it much clearer to tell what's going on, when I'm evaluating the AST.
