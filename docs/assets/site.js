/* Shared motion for every Mac Apps page: smooth scroll (Lenis), scroll progress, replaying reveals,
   pointer-following glow on cards, magnetic lean on buttons. Everything is optional: if anything fails the page just works. */
(function () {
  var reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
  var fine = matchMedia('(hover: hover) and (pointer: fine)').matches;
  var root = document.documentElement;

  // 1. smooth scroll
  function startLenis() {
    if (reduce || !window.Lenis) return;
    try {
      var lenis = new Lenis({ duration: 1.1, easing: function (t) { return 1 - Math.pow(1 - t, 4); }, smoothWheel: true });
      (function raf(t) { lenis.raf(t); requestAnimationFrame(raf); })(performance.now());
      // in-page links glide too
      document.addEventListener('click', function (e) {
        var a = e.target.closest && e.target.closest('a[href^="#"]');
        if (!a || a.getAttribute('href').length < 2) return;
        var el = a.getAttribute('href') === '#top' ? 0 : document.querySelector(a.getAttribute('href'));
        if (el === 0) { e.preventDefault(); lenis.scrollTo(0); return; }
        if (el) { e.preventDefault(); lenis.scrollTo(el, { offset: -16 }); }
      });
      window.__lenis = lenis;
    } catch (e) {}
  }
  if (!reduce) {
    var s = document.createElement('script');
    s.src = 'https://cdn.jsdelivr.net/npm/lenis@1.1.20/dist/lenis.min.js';
    s.onload = startLenis; s.onerror = function () { root.style.scrollBehavior = 'smooth'; };
    document.head.appendChild(s);
  }

  document.addEventListener('DOMContentLoaded', function () {
    // 2. reading progress
    if (false) {
      var bar = document.createElement('div'); bar.className = 'mo-progress'; document.body.appendChild(bar);
      var tick = 0;
      var upd = function () { tick = 0; var h = root.scrollHeight - innerHeight; bar.style.transform = 'scaleX(' + (h > 0 ? Math.min(1, scrollY / h) : 0) + ')'; };
      addEventListener('scroll', function () { if (!tick) tick = requestAnimationFrame(upd); }, { passive: true }); upd();
    }

    // 3. reveals that replay every time (pages that already have their own .reveal keep it)
    if (!reduce && 'IntersectionObserver' in window) {
      var own = document.querySelector('.reveal');
      var targets = own ? [] : [].slice.call(document.querySelectorAll('main > *, .grid > *, .help, .note, footer, .btns > *'));
      if (targets.length) {
        root.classList.add('mo-ready');
        targets.forEach(function (el, i) { el.classList.add('mo-reveal'); el.style.setProperty('--mo-d', Math.min(i % 6, 5) * 70 + 'ms'); });
        // grid cards fade in left to right within their own row (column order), not by page-wide index
        var gridCards = [].slice.call(document.querySelectorAll('.grid > *'));
        function stagger() {
          var rows = {};
          gridCards.forEach(function (el) { var t = Math.round(el.offsetTop); (rows[t] = rows[t] || []).push(el); });
          Object.keys(rows).forEach(function (k) { rows[k].sort(function (a, b) { return a.offsetLeft - b.offsetLeft; }).forEach(function (el, c) { el.style.setProperty('--mo-d', c * 90 + 'ms'); }); });
        }
        if (gridCards.length) { stagger(); addEventListener('resize', stagger); }
        var io = new IntersectionObserver(function (es) {
          es.forEach(function (e) { e.target.classList.toggle('mo-in', e.isIntersecting); });
        }, { threshold: 0.12, rootMargin: '0px 0px -6% 0px' });
        targets.forEach(function (el) { io.observe(el); });
      }
    }

    if (!fine || reduce) return;
    // 4. glow follows the pointer on cards
    document.addEventListener('pointermove', function (e) {
      var c = e.target.closest && e.target.closest('.card, .point');
      if (!c) return;
      var r = c.getBoundingClientRect();
      c.style.setProperty('--mx', (e.clientX - r.left) + 'px'); c.style.setProperty('--my', (e.clientY - r.top) + 'px');
    }, { passive: true });
    // 5. buttons lean toward the pointer (a few pixels, springs back)
    var lean = '.dlmac, a.btn.primary, .btn.primary';
    document.addEventListener('pointermove', function (e) {
      var b = e.target.closest && e.target.closest(lean);
      if (!b) return;
      var r = b.getBoundingClientRect();
      var x = ((e.clientX - r.left) / r.width - .5) * 6, y = ((e.clientY - r.top) / r.height - .5) * 4;
      b.style.transform = 'translate(' + x.toFixed(1) + 'px,' + y.toFixed(1) + 'px)';
    }, { passive: true });
    document.addEventListener('pointerout', function (e) {
      var b = e.target.closest && e.target.closest(lean);
      if (b && !b.contains(e.relatedTarget)) b.style.transform = '';
    });
  });
})();

// copy buttons inside terminal boxes
document.addEventListener('click', function (e) {
  var b = e.target.closest && e.target.closest('[data-copy]'); if (!b) return;
  var t = b.parentElement.querySelector('code'); if (!t) return;
  var done = function () { var o = b.textContent; b.textContent = 'Copied'; b.classList.add('done'); setTimeout(function () { b.textContent = o === 'Copied' ? 'Copy' : o; b.classList.remove('done'); }, 1600); };
  try { navigator.clipboard.writeText(t.textContent.trim()).then(done, done); } catch (err) { done(); }
});
