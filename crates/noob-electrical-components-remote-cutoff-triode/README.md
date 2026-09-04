# noob-electrical-components-remote-cutoff-triode

The gain element of a variable-mu limiter: one triode section whose grid is
wound with varying pitch, so it switches off progressively over tens of
volts instead of collapsing over a few, and whose amplification factor is
therefore a function of bias rather than a number.

It is a pure function of grid and anode voltage with no state at all —
anode current, both its partial derivatives, transconductance, plate
resistance and amplification factor — parameterised by a fitted law.

**It is not an ordinary triode model with different numbers.** The triode
models a preamplifier uses were fitted for 12AX7-class valves, which have
no remote-cutoff characteristic. The difference is in the functional form,
not the parameters, which is why this is its own component and not a
parameter set of somebody else's.

One of those models is next door, as
`noob-electrical-components-small-signal-triode`. Its law is a fixed-shape
curve whose small-signal gain is the same at every bias, so there is no bias
at which it is twenty decibels down and a control voltage applied to it
would have nothing to do. This valve's gain *is* its bias. Each crate
asserts its own half of that in a test, so the two cannot be quietly merged
by a later reader without one of the tests failing.

```toml
[dependencies]
noob-electrical-components = { git = "https://github.com/Noob-Audio-Engineering/noob-electrical-components", features = ["remote-cutoff-triode"] }
```

## The law, and the one parameter I refitted

The anode current follows Raffensperger's eight-parameter empirical fit to
General Electric's published curves for the 6386,

```text
                 p1 · Vak^p2
Ia = ───────────────────────────────────────────
     (p3 − p4·Vgk)^p5 · [ p6 + exp(p7·Vak − p8·Vgk) ]
```

**with its exponential cut-off rate refitted, because the published fit was
never constrained where a limiter actually works.** That fit is to plate
*current*, and a fit to current says nothing about its **slope**. For a
variable-mu stage the audio *is* the slope, because gain is
transconductance — and as published the slope is 42 % low at the valve's
own tabulated operating point, and its rate of fall dips and climbs again
in a way the maker's own logarithmic plot does not. Read against the plate
characteristics at 250 V, the published fit is 9 dB low at −50 V of grid
and 37 dB low at −70, which is exactly where a limiter spends its loudest
moments: a remote-cutoff valve still passing half a milliamp at −70 V *is*
the point of the type.

So `p8` moves from 0.2 to 0.131 87 with the scale `p1` renormalised, fitted
against General Electric's own plate characteristics **across the working
range and in the right topology** — one published source to another, never
to an invention. The least-squares cost falls from 20.05 to 0.09. Letting
three more parameters move buys 0.03 more and was declined, because one
changed parameter with a reason is easier to defend than four.

Both parameter sets ship, the corrected one and the published one, so the
correction can be measured against General Electric's curve rather than
asserted in the abstract.

## The parameters are per valve type, and a second type has three conditions

This crate holds a parameter set per valve type, because **an exponent read
off a datasheet is not a stable quantity**. Fitting a stretched exponential
to one valve's transconductance across the four operating conditions its
maker plots gives 1.00, 0.84, 0.71 and 0.59 — one valve, one page, a factor
of 1.7 — and every one of those fits is good to under half a decibel. A
second valve's exponent moves from 2.16 to 1.71 on nothing more than a
change of anchor points within its own single curve.

So a second valve type must be fitted **by one documented procedure, using
the same class of anchor points, on curves measured in the same topology**,
and each clause was learned by getting it wrong. The procedure moves the
answer. A fit to interior points is not a fit to endpoints. And one valve's
published plot is a cascode connection whose plate floats at the next
valve's cathode while another's is a single-section characteristic at a
fixed plate voltage, which are not the same curve of the same thing.

Without all three, two implementations agree on a number while disagreeing
about the curve, and that is worse than not sharing at all, because it
looks like corroboration.

## What is estimated, and what the accuracy figure is not

**There is no measured accuracy floor for this valve, and there cannot be
one from published data.** Exactly one datasheet for the 6386 exists,
General Electric's, so there is no second manufacturer's curve to
cross-check against and no inter-source spread to bound the model with.
What is recorded instead is a **fit residual**: 0.89 dB RMS over nine
readings taken by one person off one 1953 graph. A residual says how well a
curve was fitted, not how right the curve is, and the two are not the same
claim.

Three further things are estimates and are marked as such at the code:

- **The modern replacement's curve.** Its maker publishes the same plate
  current at the same bias and three quarters of the transconductance, so
  it is carried as the same curve stretched along the grid axis. That
  reproduces both published figures exactly and is an assumption about the
  shape everywhere else.
- **The amplification factor.** The functional form has `Vak^p2` over a
  grid-only denominator, which forces μ to rise with the plate while the
  real valve's falls. It is 10.4 at the tabulated point against a published
  17, and no choice of the eight parameters can do both. It is recorded
  rather than hidden, and a caller that divides a load against a plate
  resistance should know it before trusting the number.
- **The slope's rate of fall still turns.** The refit moved the wobble from
  0.12 dB per volt at −39 V up to 1.9 by −70, to 0.10 dB per volt at −59 V
  up to 0.5 — a fifth the size and deeper down — but it did not remove it.
  The turn comes from a power law multiplying an exponential and is a
  property of the form.

## What is not here

The part, and nothing around it. Sections in parallel, push-pull, the
cathode resistor and its bypass, the control injection and its resistors,
the time-constant network, the rectifier and its dead zone are all the
machine, and they differ from unit to unit while the valve does not.
