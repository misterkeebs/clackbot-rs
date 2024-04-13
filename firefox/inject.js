(function() {
  let wpmVisible = false;
  function checkForWpm() {
    const wpmEl = document.querySelectorAll('.wpm > [data-balloon-pos="up"]');
    if (wpmVisible) {
      if (!wpmEl || !wpmEl[0] || wpmEl[0].offsetParent === null) {
        console.log('WPM went away');
        wpmVisible = false;
      }
    } else {
      if (wpmEl && wpmEl[0] && wpmEl[0].offsetParent) {
        const wpm = wpmEl[0].textContent;
        if (wpm.indexOf('-') === -1) {
          console.log('wpm', wpm);
          wpmVisible = true;
          fetch(`https://clackbot-rs-production.up.railway.app/wpm?wpm=${wpm}`);
        }
      }
    }
    setTimeout(checkForWpm, 200);
  }

  console.log('checking for wpm...');
  checkForWpm();
})();
