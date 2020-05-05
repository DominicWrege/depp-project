function onchangeRegexRadio(target) {
    const form = document.querySelector("#RegexInputForm");
    const input = form.querySelector("#RegexInput");
    if (target.value === "UnknownRegex"){
        form.hidden = true;
        input.required = false;
        if (input.name){
            input.removeAttribute("name");
        }
    }else {
        form.hidden = false;
        input.required = true;
        input.setAttribute("name", "regex");
    }
}

function goBack(exerciseID){
    this.window.location.href = this.window.location.href.replace(/assignment\/\S+/, `exercise/${exerciseID}`);
}

window.onload = (e) => {
   document.querySelector(".radio-regex>input:checked").click();
    let previousUrl = document.referrer;
    console.log(previousUrl);
};
