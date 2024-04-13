# Mockagen

The sequencer works but is messy.
Some functions take RuleData, others Pair<'_, Rule>, others AnnotatedPairs<'_>. It's confusing.
See if you can convert Pairs<> into a simpler type like Vec<Pair<'_, Rule>> and see if you can turn
Pair<'_, Rule> into RuleData or something like it.
I want everything to have the same type signature so that the code is a little more homogenous & consistent


What sort of type signature should I create for the evaluator?

In the last implementation, I returned a hash of functions with their names as the key.
Of course the functions that have a USING clause must accept arguments, too.
Presumably that means I'd have to order the rules by their dependencies?

In my last method, I generated and returned lambdas. That's a very functional approach.
What would it be like if I used structs this time to describe what functions to call?
Would that be simpler? Would it be faster?
I'm not confident it would be, tbh.
Although it'd definitely be easier to serialise.
