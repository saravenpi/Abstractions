(function () {
  const track = document.querySelector(".shots .track");
  if (!track) return;
  const nav = document.querySelector(".shots-nav");
  const now = document.getElementById("shot-now");
  document.getElementById("shot-total").textContent = track.children.length;
  const slides = Array.from(track.children);
  const calm = matchMedia("(prefers-reduced-motion: reduce)");

  function index() {
    const left = track.getBoundingClientRect().left;
    const gap = slides.map((slide) => Math.abs(slide.getBoundingClientRect().left - left));
    return gap.indexOf(Math.min.apply(null, gap));
  }

  function go(to) {
    const slide = slides[Math.max(0, Math.min(slides.length - 1, to))];
    track.scrollTo({ left: track.scrollLeft + slide.getBoundingClientRect().left - track.getBoundingClientRect().left, behavior: calm.matches ? "auto" : "smooth" });
  }

  function mark() {
    now.textContent = index() + 1;
  }

  nav.hidden = false;
  nav.addEventListener("click", function (event) {
    const button = event.target.closest("button[data-step]");
    if (button) go(index() + Number(button.dataset.step));
  });
  track.addEventListener("keydown", function (event) {
    if (event.key === "ArrowRight") go(index() + 1);
    else if (event.key === "ArrowLeft") go(index() - 1);
    else if (event.key === "Home") go(0);
    else if (event.key === "End") go(slides.length - 1);
    else return;
    event.preventDefault();
  });
  track.addEventListener("scroll", mark, { passive: true });
  mark();
})();
