(function () {
  const URL = "https://clackbot-rs-production.up.railway.app";
  // const URL = "http://localhost:8080";

  let wpmVisible = false;
  let currentWpm = 0;

  function checkLiveWpm() {
    const wpmEl = document.querySelectorAll("#miniTimerAndLiveWpm > .wpm");
    if (wpmEl?.[0]) {
      const wpm = wpmEl[0].textContent;
      if (wpm !== currentWpm) {
        console.log("sending live wpm", wpm);
        console.log(`${URL}/setLiveWpm?wpm=${wpm}`);
        fetch(`${URL}/setLiveWpm?wpm=${wpm}`);
        currentWpm = wpm;
      }
    }
    setTimeout(checkLiveWpm, 200);
  }

  function checkForWpm() {
    const wpmEl = document.querySelectorAll('.wpm > [data-balloon-pos="up"]');
    if (wpmVisible) {
      if (!wpmEl || !wpmEl[0] || wpmEl[0].offsetParent === null) {
        console.log("WPM went away");
        currentWpm = 0;
        wpmVisible = false;
      }
    } else {
      if (wpmEl?.[0]?.offsetParent) {
        const wpm = wpmEl[0].textContent;
        if (wpm.indexOf("-") === -1) {
          console.log("wpm", wpm);
          wpmVisible = true;
          fetch(`${URL}/finishWpm?wpm=${wpm}`);
        }
      }
    }
    setTimeout(checkForWpm, 200);
  }

  console.log("checking for wpm...");
  checkForWpm();
  checkLiveWpm();
})();
