(function () {
  const canvas = document.getElementById("field");
  const context = canvas && canvas.getContext("2d");
  const still = matchMedia("(prefers-reduced-motion: reduce)");
  const ramp = " .:-=+*#%@";
  const interval = 1000 / 12;
  const grid = { cols: 0, rows: 0, width: 10, height: 20, calmFrom: 0, calmTo: 0, depth: 0.85, cells: new Uint8Array(0) };
  let last = 0;
  let frame = 0;

  function measure() {
    const ratio = Math.min(devicePixelRatio || 1, 2);
    const column = document.querySelector("main").getBoundingClientRect();
    let size = Math.max(14, Math.min(22, innerWidth / 70));
    while ((innerWidth / (size * 0.6)) * (innerHeight / (size * 1.2)) > 5000) size += 1;
    canvas.width = Math.round(innerWidth * ratio);
    canvas.height = Math.round(innerHeight * ratio);
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    context.textBaseline = "top";
    context.fillStyle = "rgba(255, 255, 255, 0.2)";
    grid.width = Math.round(size * 0.6 * ratio) / ratio;
    grid.height = Math.round(size * 1.2 * ratio) / ratio;
    context.font = grid.width / 0.6 + 'px "undefined", monospace';
    grid.cols = Math.ceil(innerWidth / grid.width);
    grid.rows = Math.ceil(innerHeight / grid.height);
    grid.cells = new Uint8Array(grid.cols * grid.rows);
    grid.calmFrom = column.left / grid.width - 2;
    grid.calmTo = column.right / grid.width + 2;
    grid.depth = column.width > innerWidth * 0.75 ? 0.8 : 0.85;
  }

  function calm(x) {
    const edge = Math.min(x - grid.calmFrom, grid.calmTo - x) / 5;
    return 1 - grid.depth * Math.max(0, Math.min(1, edge));
  }

  function level(x, y, time) {
    const a = Math.sin(x * 0.045 + time * 0.21);
    const b = Math.sin(y * 0.09 - time * 0.17);
    const c = Math.sin((x * 0.6 + y) * 0.05 + time * 0.13);
    const d = Math.sin(Math.hypot(x * 0.6 - 30, y - 12) * 0.11 - time * 0.19);
    return (a + b + c + d + 4) / 8;
  }

  function draw(time) {
    for (let y = 0; y < grid.rows; y += 1) {
      for (let x = 0; x < grid.cols; x += 1) {
        const value = level(x, y, time) ** 2 * calm(x);
        const index = Math.min(ramp.length - 1, Math.floor(value * ramp.length));
        const cell = y * grid.cols + x;
        if (grid.cells[cell] === index) continue;
        grid.cells[cell] = index;
        context.clearRect(x * grid.width, y * grid.height, grid.width, grid.height);
        context.fillText(ramp[index], x * grid.width, y * grid.height);
      }
    }
  }

  function tick(now) {
    frame = requestAnimationFrame(tick);
    if (now - last < interval) return;
    last = now;
    draw(now / 1000);
  }

  function start() {
    cancelAnimationFrame(frame);
    measure();
    draw(still.matches ? 40 : performance.now() / 1000);
    if (still.matches || document.hidden) return;
    frame = requestAnimationFrame(tick);
  }

  if (!context) return;
  addEventListener("resize", start);
  document.addEventListener("visibilitychange", start);
  still.addEventListener("change", start);
  document.fonts.load('16px "undefined"').then(start, start);
})();
