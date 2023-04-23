import os
import time
import urllib.request
from bs4 import BeautifulSoup
import requests
import requests.utils
import consts
import math


def obey_robots_txt():
    import urllib.robotparser
    rp = urllib.robotparser.RobotFileParser()
    rp.set_url(f"{consts.WEBSITE}/robots.txt")
    rp.read()
    return rp


class Crawler:

    def __init__(self):
        self.rp = obey_robots_txt()
        self.delay = self.rp.crawl_delay(consts.UA)
        if self.delay is None:
            rrate = self.rp.request_rate(consts.UA)
            if rrate is not None:
                self.delay = math.ceil(rrate.seconds / rrate.requests)
        if self.delay is None:
            print(f"cannot find Crawl-Delay or Request-Rate in robots.txt for UA '{consts.UA}', set to 3 as default")
            self.delay = 3
        print(f"Crawler init with parameters: website {consts.WEBSITE}, delay {self.delay}")

    def get(self, url, params=None, **kwargs):
        headers = requests.utils.default_headers()
        headers["User-Agent"] = consts.UA
        kwargs["headers"] = headers

        time.sleep(self.delay)

        if not self.rp.can_fetch(consts.UA, url):
            print(f"Crawler cannot fetch url({url}) which is disallowed in robots.txt")
            return None
        res = requests.get(url, params, **kwargs)
        if res.ok:
            return res.text
        print(f"Crawler get url({url}) failed, response: {res}")
        return None


class Parser:

    def __init__(self, crawler):
        self.crawler = crawler

    def parse_products_page(self):
        html = self.crawler.get(consts.PRODUCTS_PAGE)
        if html is None:
            return None
        soup = BeautifulSoup(html)
        products_div = soup.find(role="main").find_all("div")
        products = []
        for div in products_div:
            try:
                products.append((div.span.text, div.img["alt"], "https:" + div.img["src"]))
            except Exception:
                continue
        return products


class Resolver:

    def __init__(self, parser):
        self.parser = parser

    def resolver_to_file(self):
        products = self.parser.parse_products_page()
        if products is None:
            return
        path = os.getcwd() + "/images/"
        os.makedirs(path)
        for product in products:
            try:
                urllib.request.urlretrieve(product[2],
                                           filename=(path + product[1] + "_" + product[0] + ".png").replace(" ", "_"))
            except Exception:
                continue


def main():
    crawler = Crawler()
    parser = Parser(crawler)
    resolver = Resolver(parser)
    resolver.resolver_to_file()


if __name__ == '__main__':
    main()
