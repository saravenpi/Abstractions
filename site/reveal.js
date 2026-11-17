(function () {
  const items = document.querySelectorAll(".reveal");
  if (!("IntersectionObserver" in window) || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
  const observer = new IntersectionObserver(
    function (entries) {
      entries.filter((entry) => entry.isIntersecting).forEach(function (entry) {
        entry.target.classList.add("in");
        observer.unobserve(entry.target);
      });
    },
    { rootMargin: "0px 0px -60px 0px", threshold: 0.05 },
  );
  document.documentElement.classList.add("js");
  items.forEach((item) => observer.observe(item));
})();
