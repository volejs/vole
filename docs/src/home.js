const canvas = document.querySelector(".hero-atmosphere");
const context = canvas?.getContext("2d", { alpha: false });
const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");

// Render the soft horizontal light bands as a grainy bitmap, without an image download.
if (context) {
  canvas.width = 768;
  canvas.height = 480;
  const pixels = context.createImageData(canvas.width, canvas.height);
  const bands = [
    { y: .23, width: .025, color: [3, 15, 18], bend: .009 },
    { y: .43, width: .014, color: [3, 20, 24], bend: .012 },
    { y: .51, width: .018, color: [7, 27, 15], bend: -.01 },
    { y: .77, width: .027, color: [4, 12, 22], bend: .012 },
    { y: .94, width: .016, color: [6, 20, 19], bend: -.007 },
  ];
  let seed = 9173;
  for (let x = 0; x < canvas.width; x++) {
    const u = x / canvas.width;
    const intensity = .1 + Math.pow(u, 1.7) * .9;
    const ribbonPositions = bands.map((band) => band.y + Math.sin(u * 7) * band.bend);
    for (let y = 0; y < canvas.height; y++) {
      seed = (Math.imul(seed, 1664525) + 1013904223) | 0;
      const noise = ((seed >>> 24) / 255 - .5) * 6;
      const v = y / canvas.height;
      const rgb = [11 + noise, 16 + noise, 19 + noise];
      for (let i = 0; i < bands.length; i++) {
        const distance = (v - ribbonPositions[i]) / bands[i].width;
        const light = Math.exp(-distance * distance * .5) * intensity;
        for (let channel = 0; channel < 3; channel++) rgb[channel] += light * bands[i].color[channel];
      }
      const offset = (y * canvas.width + x) * 4;
      pixels.data[offset] = rgb[0];
      pixels.data[offset + 1] = rgb[1];
      pixels.data[offset + 2] = rgb[2];
      pixels.data[offset + 3] = 255;
    }
  }
  context.putImageData(pixels, 0, 0);
}

const windowElement = document.querySelector(".capability-window");
const track = document.querySelector(".capability-track");
if (track && windowElement) {
  const originalRows = [...track.children];
  const count = originalRows.length;
  for (const row of originalRows) {
    const clone = row.cloneNode(true);
    clone.setAttribute("aria-hidden", "true");
    clone.querySelector("a").tabIndex = -1;
    track.append(clone);
  }
  const rows = [...track.children];
  let active = 4;
  let timer;
  let visible = true;
  let hovered = false;
  let focused = false;
  let moving = false;

  const paint = () => {
    windowElement.style.setProperty("--index", active);
    rows.forEach((row, index) => row.toggleAttribute("data-active", index === active));
  };
  const advance = async () => {
    if (moving) return;
    moving = true;
    active++;
    paint();
    await new Promise(requestAnimationFrame);
    await Promise.allSettled(track.getAnimations().map((animation) => animation.finished));
    if (active >= count + 4) {
      track.classList.add("is-resetting");
      active -= count;
      paint();
      await new Promise(requestAnimationFrame);
      await new Promise(requestAnimationFrame);
      track.classList.remove("is-resetting");
    }
    moving = false;
  };
  const updateTimer = () => {
    clearInterval(timer);
    if (visible && !hovered && !focused && !document.hidden && !reducedMotion.matches) {
      timer = setInterval(advance, 2600);
    }
  };
  windowElement.addEventListener("pointerenter", () => { hovered = true; updateTimer(); });
  windowElement.addEventListener("pointerleave", () => { hovered = false; updateTimer(); });
  track.addEventListener("focusin", (event) => {
    focused = true;
    const index = rows.indexOf(event.target.closest("li"));
    if (index >= 0) { active = index; paint(); }
    updateTimer();
  });
  track.addEventListener("focusout", (event) => {
    focused = track.contains(event.relatedTarget);
    updateTimer();
  });
  document.addEventListener("visibilitychange", updateTimer);
  reducedMotion.addEventListener("change", updateTimer);
  new IntersectionObserver(([entry]) => {
    visible = entry.isIntersecting;
    updateTimer();
  }).observe(windowElement);
  paint();
  updateTimer();
}
