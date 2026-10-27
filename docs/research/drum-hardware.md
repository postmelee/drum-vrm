# Acoustic drum mounting hardware

Internal research for the generated hardware topology and its visual inspection.
The [equipment/contact contract](../../specs/architecture/equipment-contact.md)
owns implemented behavior; this note separates manufacturer evidence from our
engineering choices. Sources and exploded diagrams were inspected on 2026-10-27.

## Evidence and dimensions

| ID | Primary source | Observed structure or published dimension |
| --- | --- | --- |
| H01 | TAMA [MTH600 product](https://www.tama.com/usa/products/detail/mth600.html) and [parts diagram](https://hoshinogakki.co.jp/pdf/tama/world/support/parts/hardware/tom_holder/MTH600.pdf), one page, production revision labels beginning June 2006 | A 25.4 mm bass-mounted post carries two independently clamped ball joints. Each ball carries an L-shaped rod; the diagram lists a 10.5 mm rod lock and separate post lock. The short lateral section bends into an upward insertion section for the external tom receiver. Wing nuts clamp the sockets. |
| H02 | TAMA [HC83BW product](https://www.tama.com/usa/products/detail/hc83bw.html) and [parts diagram](https://hoshinogakki.co.jp/pdf/tama/world/support/parts/hardware/cymbal_stand/HC83BW.pdf), one page, production revision November 2014 | Three telescoping tube diameters: 28.6, 22.2 and 19.1 mm. The diagram labels component lengths 430, 490 and 540 mm; these are not three fully exposed sections. The boom is 450 mm. A separate boom clamp and end tilter support a sleeve, felts and retainer. The tripod has double braces and rubber feet. Product height range is 820–2,100 mm. |
| H03 | Pearl [BC-2030 instruction manual](https://pearldrum.com/sites/default/files/image_folder/SUPPORT/MANUAL/HARDWARE/BOOM_STANDS/BC-2030_2023_manual%28E%29PDF_A.pdf), November 2023, pages 1 and 4 | Separate base, centre and upper tubes, joints, boom and cymbal tilter form the assembly. The stability instruction aligns one tripod leg with the boom direction and requires checking the resulting load and readjusting boom length/direction when necessary. |
| H04 | Steve La Cerra, Yamaha, [The Modern Drum Set, Part 5: Cymbals and Hardware](https://hub.yamaha.com/drums/stage/cymbals-and-hardware/), 2020-04-21 | Straight and boom stands are both conventional; booms separate cymbal reach from floor placement. Both single and double braces are legitimate. Rack toms can use a bass mount, separate stand or rack. Snare support is a basket, with height and tilt chosen for the player. |
| H05 | Steve La Cerra, Yamaha, [The Modern Drum Set, Part 3: Toms](https://hub.yamaha.com/drums/studio/part-3-toms/), 2020-02-20 | Shell-mounted floor legs and stand/rack mounting are distinct hardware arrangements. Common floor diameters include 14, 16 and 18 inches; diameter and depth vary independently. A different support arrangement does not by itself establish better player clearance. |
| H06 | Steve La Cerra, Yamaha, [The Modern Drum Set, Part 4: Foot Pedals](https://hub.yamaha.com/drums/stage/the-modern-drum-set-part-4-foot-pedals/), 2020-03-24; Yamaha [basic position](https://www.yamaha.com/en/musical_instrument_guide/drums/play/) | A conventional hi-hat links its footboard to the pull rod; rotating legs help nearby pedal placement. Left foot operates the hi-hat, right foot the kick. Pedal pitch, placement on the floor, and tripod rotation are different adjustments. Neither source prescribes one universal floor yaw angle. |
| H07 | TAMA [Single Tom Adapter: MC69 / MTH900AS](https://www.tama.com/eu/products/detail/single_tom_adapter.html) | MC69 clamps to a 15.9–28.6 mm diameter pipe and uses an Omni-ball angle joint. MTH900AS combines a ball joint and L-shaped rod to attach a single tom to a stand. The page does not publish rod length, floor-tom mass capacity or a safe extension for a shared cymbal/tom load. |

These examples establish common mechanical relationships, not one mandatory kit
design. TAMA's L-rod receiver family is a coherent reference for this procedural
kit. Other manufacturers use different rod/receiver axes; combining their visible
parts without matching those axes would produce an implausible hybrid.

## Structural interpretation

A rack tom should not be held by a diagonal line terminating at the centre of its
bottom head. For the chosen mounting family, the load path is:

```text
bass-drum bracket → vertical post → twin socket yoke
                                   ├→ ball → L-rod → external tom receiver
                                   └→ ball → L-rod → external tom receiver
```

The receiver belongs outside the shell, connected to a visible bracket or
suspension cradle. Rods must not pass through a head or shell to reach it. The
shared post is one assembly, even when both toms refer to it. Its socket positions,
rod endpoints and receiver axes must agree after tilt and layout fitting.

A boom stand has three distinct adjustments: telescoping height, boom direction
and extension, and cymbal tilt. It is not a continuous bent tube. Render a clamp at
the mast/boom pivot, a straight boom passing through it with a rear tail, and an
end tilter whose short cymbal stud follows the cymbal normal. The seat, sleeve,
felts and retainer remain distinct from the playing surface. A free-standing
tripod should show joints, braces and feet rather than three unrelated diagonal
lines. The floor tom instead has shell-mounted legs; the snare has a basket.

A suspended low tom is a separate mounting family. It attaches to a compatible
stand tube through a clamp and tom receiver; the cymbal disc does not bear the
tom's load. Both instrument load paths must reach the same supported base, with
bounded rods and clear shells/pedals. Removing floor legs and drawing a connector
is insufficient. A diagram showing this option is not evidence that shell legs
are inherently incorrect or that the particular shared stand can carry any load.

H07 supports a coherent optional stand-clamped topology for this project:
named host mast → clamp/socket → angle joint → L-shaped rod → external shell
receiver. It does not justify replacing every floor-tom leg arrangement. Keep the
three-leg default and the suspended option separate from shell appearance presets;
the selected physical connection must survive fitting, serialization and rendering.

For the reference kit, first reserve space for both seated legs and the pedal
strokes. Place the floor shell and its legs outside that envelope while preserving
hand reach. An inward-facing heel and outward-facing toe can align the hi-hat
board with the seated leg; derive this direction from common body geometry rather
than imposing a fixed photograph angle. The same direction must orient the foot,
hinge, collision box, mast and linkage. Rotating only the visible pedal would
misrepresent both contact and clearance.

## Engineering choices and limits

The following are project design decisions, not manufacturer specifications:

- Choose semantic part roles and one numerical assembly graph for rendering and
  clearance. A clamp, tube and rubber foot may render differently while remaining
  within their corresponding collision envelope. Detail must not create an
  untested protrusion outside that envelope.
- Use the published tube/rod diameters as scale references. Published tube
  component lengths require overlap at each telescoping joint. Exposed lengths,
  insertion margins, joint size, L-rod reach and bracket dimensions need bounded
  project defaults; the reviewed MTH600 source does not specify rod lengths.
- Orient one boom-stand foot toward the boom as H03 recommends. Limit extension
  and report unsupported arrangements; this rule alone is not a centre-of-mass or
  structural-load calculation. A decorative counterweight is not a substitute.
- The generated boom permits up to 450 mm from pivot to tilter plus a 100 mm
  rear tail. This is our compact routing bound, not H02's published total rod
  length or an exact product replica. The 300 mm tripod radius, 380 mm tom-arm
  reach and 300 mm insertion rise are also engineering bounds.
- A snare basket uses a separate compact base with 225 mm radius rather than
  inheriting the wider boom-stand footprint. H04 recommends compact snare bases
  but does not establish this numerical radius. Its initial rear-facing leg leaves
  a forward sector for pedal access; an explicit world-space tripod phase can be
  retained when body-aware fitting finds a clearer orientation. No leg or pedal
  collision is waived by selecting this mechanical family.
- For the optional stand-clamped floor tom, use H07's compatible pipe diameters;
  a 50 mm socket offset, 25–380 mm horizontal arm and 80–300 mm vertical rise are
  our search bounds. The named host is routed first. Both load projections must
  lie inside its tripod triangle, while the arm clears the shell and unrelated
  hardware. This is a conservative geometric gate, not a mass, torque, clamp
  friction or manufacturer load-rating calculation. An infeasible route remains
  unresolved; it cannot silently regain floor legs or an arbitrary connector.
- Compact/avatar fitting may change instrument placement, but cannot silently
  stretch rods or move a receiver through a shell. If a coherent mounting family
  cannot reach the fitted layout, retain an unresolved result or explicitly
  select a different supported mounting topology.
- Treat the direct hi-hat shaft and footboard as one assembly. A lateral offset
  requires an explicitly modelled remote mechanism, rather than an invisible
  connection. Pedal clearance and player access remain constraints.
- Share intentional-joint identities. Exempting every pair owned by the same
  instrument can hide a boom crossing its shell or a brace entering its pedal.
  Only the intended local attachment overlap should be exempted.

## Feasibility checks from multiple views

An attractive front view cannot establish a feasible three-dimensional assembly.
Use the same final measured instrument geometry and hardware for all checks.

| View or measurement | What to inspect |
| --- | --- |
| Front | Both rack toms connect to the shared holder; branches reach exterior receivers without crossing neighbouring shells. Snare basket sits below the shell and its fingers meet the rim support region. |
| Top | Tom/snare playing regions remain usable. Boom bases and feet clear pedals, nearby stands and the player's access. A cymbal may overlap a tom in projection; this alone is not a collision. |
| Both sides | Telescoping joints overlap, toms clear the bass shell, booms clear drum bodies, and the tilted cymbal has vertical clearance above any projected overlap. Look for unsupported floating receivers. |
| Oblique/detail | Ball/socket, L-rod/receiver and boom/tilter axes actually meet. Braces terminate at hinges, feet touch the ground, and studs pass through the intended cymbal hole. |
| Numerical | Test body/body, support/body, support/pedal and unrelated support/support clearance. Check bounded reach and intended joints separately; display unresolved hardware routes. |
| Motion | Check the complete stick, including its middle and butt, through preparation, impact, recovery and transfer. Static head clearance does not prove a usable moving-stick corridor or safe cymbal sweep. |

Repeat the views for the default kit and accepted fitted layouts. Rendering and
numeric checks are complementary: capsule/box clearance does not establish exact
mesh clearance, clothing clearance, load stability or acoustic plausibility.
Record observed results with their candidate commit in a verification report,
rather than replacing these general checks with a dated claim of completion.

## Attribution and reuse boundary

TAMA/Hoshino Gakki, Pearl and Yamaha retain rights in their manuals, photographs,
drawings, names and product designs. These links and independently written
summaries are research references. No manufacturer CAD, texture, photograph or
diagram is incorporated into the app or public repository by this research.
Downloaded inspection copies remain local; no open asset license was established.
The application uses independently authored generic procedural geometry and does
not claim to reproduce, certify or endorse a named manufacturer's product.
Repository MIT licensing does not license the referenced source material.
