function sendQuery() {

    let query = document.getElementById("search_bar").value

    console.log("query" + query);

    fetch("api/search", {
        method: 'POST',
        body: query
    })
        .then(response => response.json())
        .then(response => {
        let searchList = document.getElementById("search_list")
        searchList.innerHTML = null;
        response.map(file => [file, document.createElement("p")])
            .forEach(arr => {
                arr[1].innerHTML = arr[0];
                searchList.append(arr[1]);
            });
    })
}

document.getElementById("search_bar").addEventListener("keyup", function(event) {
    event.preventDefault();
    document.getElementById("search_btn").click();
    if (event.key == "Enter") {
    }
})
