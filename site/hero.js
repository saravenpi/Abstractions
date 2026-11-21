(function () {
  const SLIDES = [
    // hero:start
    // hero:end
  ];
  const HOLD = 6000;
  const FADE = 1500;
  const SIZES = [1280, 1920];

  const show = document.getElementById("show");
  if (!show || SLIDES.length < 2) return;
  if (matchMedia("(prefers-reduced-motion: reduce)").matches) return;
  const layers = Array.from(show.querySelectorAll(".layer"));
  if (!layers.length) return;
  if (layers.length < 2) {
    const spare = document.createElement("figure");
    spare.className = "layer";
    spare.setAttribute("aria-hidden", "true");
    spare.innerHTML = '<picture><source type="image/webp"><img alt="" decoding="async"></picture>';
    show.append(spare);
    layers.push(spare);
  }
  const hero = show.parentElement;
  let front = 0;
  let index = 0;
  let timer = 0;
  let seen = true;
  let awake = !document.hidden;
  let busy = false;

  function srcset(base, v, ext) {
    return SIZES.map((w) => base + "-" + w + "." + ext + "?v=" + v + " " + w + "w").join(", ");
  }

  function fill(layer, slide) {
    const picture = layer.querySelector("picture");
    const image = picture.querySelector("img");
    picture.querySelector("source").srcset = srcset(slide.base, slide.v, "webp");
    image.sizes = "100vw";
    image.srcset = srcset(slide.base, slide.v, "jpg");
    image.src = slide.base + "-" + SIZES[0] + ".jpg?v=" + slide.v;
    image.alt = slide.alt;
    return image;
  }

  function settle() {
    return new Promise((resolve) => setTimeout(resolve, FADE));
  }

  function swap(next) {
    const from = layers[front];
    const to = layers[next];
    to.classList.remove("go");
    void to.offsetWidth;
    to.classList.add("go", "on");
    to.removeAttribute("aria-hidden");
    from.classList.remove("on");
    from.setAttribute("aria-hidden", "true");
    front = next;
    return settle().then(function () {
      if (layers[front] !== from) from.classList.remove("go");
    });
  }

  function advance() {
    if (busy) return Promise.resolve();
    busy = true;
    const target = (index + 1) % SLIDES.length;
    const image = fill(layers[1 - front], SLIDES[target]);
    return image
      .decode()
      .then(
        function () {
          index = target;
          return swap(1 - front);
        },
        function () {
          index = target;
        },
      )
      .then(function () {
        busy = false;
        queue();
      });
  }

  function queue() {
    clearTimeout(timer);
    if (seen && awake) timer = setTimeout(advance, HOLD);
  }

  function state() {
    show.classList.toggle("idle", !(seen && awake));
    queue();
  }

  new IntersectionObserver(function (entries) {
    seen = entries[entries.length - 1].isIntersecting;
    state();
  }).observe(hero);
  document.addEventListener("visibilitychange", function () {
    awake = !document.hidden;
    state();
  });
  queue();
})();
