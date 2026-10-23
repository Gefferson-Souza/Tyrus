'use strict';
// Shim de determinismo carregado por --require nos dois lados (relógio, RNG, pid).
// Sem ele o oráculo compararia a hora e o acaso da máquina, não o comportamento.
const EPOCH_MS = 1_700_000_000_000;
const FIXED_PID = 4242;
const RNG_SEED = 0x9e3779b9;

const RealDate = Date;
const realNow = RealDate.now.bind(RealDate);

globalThis.Date = new Proxy(RealDate, {
  construct(target, args, newTarget) {
    return Reflect.construct(target, args.length === 0 ? [EPOCH_MS] : args, newTarget);
  },
  apply() {
    return new RealDate(EPOCH_MS).toString();
  },
  get(target, prop) {
    if (prop === 'now') return () => EPOCH_MS;
    return Reflect.get(target, prop, target);
  },
});

let state = RNG_SEED >>> 0;
Math.random = function random() {
  state = (state + 0x6d2b79f5) >>> 0;
  let t = state;
  t = Math.imul(t ^ (t >>> 15), t | 1);
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};

Object.defineProperty(process, 'pid', { value: FIXED_PID, configurable: true });

// Litmus de perturbação: com TYRUS_SHIM_PERTURB_MS, cada registro de timer espera
// esse tempo real antes de voltar; uma ordem que depende de espera >= 10 ms muda.
const perturbMs = Number(process.env.TYRUS_SHIM_PERTURB_MS || 0);
if (perturbMs > 0) {
  const spin = () => {
    const end = realNow() + perturbMs;
    while (realNow() < end) { /* espera ocupada */ }
  };
  for (const name of ['setTimeout', 'setInterval']) {
    const original = globalThis[name];
    const wrapper = function perturbed(...args) {
      const handle = original.apply(this, args);
      spin();
      return handle;
    };
    // Preserva `util.promisify.custom` e demais propriedades próprias do original.
    Object.defineProperties(wrapper, Object.getOwnPropertyDescriptors(original));
    globalThis[name] = wrapper;
  }
}
