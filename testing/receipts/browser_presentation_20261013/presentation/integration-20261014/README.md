# Shared-main integration, 2026-10-14

The browser presentation lane (`webgl-presentation`, closed PR #8) was merged
onto Mere main on the ThinkPad (AMD Radeon Renoir). Netrender's lane
(`cbdbf1e6`, closed Netrender PR #1) was first merged onto Netrender main as
`68bcddc06`, and Mere pins that one revision.

Native: 336 passed across Rootstock, Mesquite, the winit host and the web host
(11 existing ignores); the WebGL portable test passes 5; Graphshell passes 513
(7 ignores). Dramatis UI, the moot resource host, Tenant and the wasm web host
build. Netrender's sparse/WebGL set passes 28.

Browser, actual packaged viewer: Chrome WebGL2 on AMD, Auto with WebGPU
disabled, Firefox and software SwiftShader each pass 32 controls; Chrome native
Vulkan WebGPU passes 26. Same results as the lane's own receipts.

Owner limits, unchanged and still unmet:

- Wasm delivery: gzip6 3,661,337 bytes (gzip9 3,615,553) against the
  3,407,872 cap. Glue 18,445, mount 592 and history 6,490 pass.
- 125% display scale, WebGL2 versus WebGPU: RGB RMS 1.26451 (before loss) and
  1.19021 (after restore) against 1.0; alpha matches; the resized comparison
  passes at 0.54310. Normal scale passes at 0.25225/0.29989/0.29905, max 11.

`compare.py` is the comparator used here; it reproduces the lane's recorded
values. Exact figures, renderer identities and bundle hash are in
`integration.json`; raw captures stay under
`Code/testing/mere/webgl-integration-20261014`.
