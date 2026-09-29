Decisions/Assumptions:


- `if` rules are checked in order, and the first matching rule decides the outcome
- returned outcome should be also in the grammar for early returns
- should be able to return true/false/variant name/identifier
- we need backward compatibility so do not touch define new expressions, new rule has `if ... then ...;` syntax and if that is missing it means `if ... then true`
- no rule matches mean `False`
- there is a default value at the end, which can also be a variant or True/False
- old rules are fallthrough rules (like in LD = `if ... then true`)


IDEA:
we need a compile rule fn where we pass the variants (come from legacy toggle.variant), sth like:
and if the flag does not have the variant we want to return,  -> `False` also

```rust
compile_rule_with_variants(rule, &["blue", "red", "green", "control", "variant1"])
```


QUESTION:
- why removed flag level variant? - LD does it
- we need to define multiple strategies, a bit cumbersome?

## 1. old rules (should be unchanged and work as is. e.g.:)

```
50% sticky on user_id with group_id of "my-flag"
```

## 2. early return with `false`

```
if user_id in ["bad_group"] then false; 50% sticky on user_id
```

`bad_group` is always `Flase`. others go to the 50% rollout

```
if user_id in ["bad_group"] then false; if remote_address in_cidr ["10.0.0.0/8"] then false; true
```

Disabled rules can be stacked. `bad_group` and any address in `10.0.0.0/8` are `False`, and everyone else is `True`.

## 3. early return with `true`

```
if user_id in ["beta-user"] then true; 10% sticky on user_id
```

`beta-user` is always `True`, even if they're outside the 10% rollout.

## 4. only `if` rules, with no fallthrough

```
if environment in ["dev"] then false; if context["plan"] in ["pro"] then true;
```

If no rule matches, the result is `False`. Here `plan=pro` is `True`, and both `dev` and all other users are `False`.

## 5. Fixed variant (individual targeting in LD)

```
if user_id in ["beta-user"] then variant "blue"; true
```

`beta-user` gets `Variant("blue")`. Everyone else gets `True`

```
if true then variant "blue";
```

everyone gets `blue`.

## 6. Variant split

```
if true then variants ["blue": 500, "red": 500];
```

A 50/50 split with default stickiness (`user_id`, then `session_id`, then random). The group ID defaults to the flag name.

```
if true then variants ["control": 200, "blue": 300, "green": 500] sticky on context["customcontex"] | user_id with group_id of "group1";
```

stickiness with a fallback, plus an explicit group ID. All users with the same `customcontex` get the same variant.

## 7. Split for a segment only

```
if context["plan"] in ["pro"] then variants ["blue": 1, "red": 1]; true
```

`pro` users are split between `blue` and `red`. Everyone else gets `True`.

## 8. Two layers: rollout, then split

```
if 90% sticky on user_id with group_id of "my-flag" then variants ["control": 50, "blue": 50];
```

this is what we have now: the rollout decides who's in, and the variants divide the users. users outside the rollout, or without a `user_id`, get `False`.

## 9. Complex conditions

```
if (context["plan"] in ["pro"] or user_id in ["beta-user"]) and !environment in ["dev"] then variant "blue";
```

The condition can be any expression. can use parentheses here to add arbitrary nested structures


## 10. A variant override as an `if` rule

```
if user_id in ["132", "61"] then variant "variant1"; true
```

## 11. everything together

```
if user_id in ["bad"] then false; if user_id in ["beta-user"] then variant "blue"; if context["plan"] in ["pro"] then variants ["blue": 50, "red": 50]; 25% sticky on user_id
```

The rules are checked in this order:

1. Deny rule
2. Individual target
3. Segment split
4. Fallthrough rollout


## 12. Negating a group

```
!(user_id in ["bad"] or remote_address in_cidr ["10.0.0.0/8"])
```

`bad` and any address in `10.0.0.0/8` are `False`, everyone else is `True`.

```
!(context["plan"] in ["pro"] and environment in ["dev"])
```

only `pro` users in `dev` are `False`.

```
if !(user_id in ["beta-user"] or context["plan"] in ["pro"]) then false; true
```

deny everyone who is neither `beta-user` nor `pro`.

```
if !(context["plan"] in ["pro"] and environment in ["dev"]) then variant "blue"; true
```

works in `if` conditions too.

```
!(!(user_id in ["42"]))
```

groups can be nested, same as `user_id in ["42"]`.

```
!(true) or user_id in ["42"]
```

`!` should bind tighter than `and`/`or`, so this is `(!true) or user_id in ["42"]`.

### Negated segment

A segment is its constraints AND-ed together, so NOT segment is `!(c1 and c2 and ...)`. The upgrader could emit it next to the strategy's own constraints:

```
(true and (50% sticky on user_id and !(context["plan"] in ["pro"] and environment in ["dev"])))
```

the same only single-constraint `!` (what works without the change):

```
(true and (50% sticky on user_id and (!context["plan"] in ["pro"] or !environment in ["dev"])))
```

### not allowed

- `!!(true)`: only one `!` in front of a group, same as for a single constraint.

## Rules that should fail to compile and return False
- non existing variants
- invalid splits: sum not 1