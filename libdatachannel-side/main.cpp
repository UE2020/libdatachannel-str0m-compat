#include "json.hpp"
#include <chrono>
#include <iostream>
#include <memory>
#include <rtc/rtc.hpp>
#include <thread>

using nlohmann::json;

int main() {
    try {
        rtc::InitLogger(rtc::LogLevel::Warning);

        rtc::Configuration config;
        config.iceServers.emplace_back("stun.l.google.com:19302");
        auto pc = std::make_shared<rtc::PeerConnection>(config);
        auto channel = pc->createDataChannel("test");

        pc->onStateChange(
            [](rtc::PeerConnection::State state) {
                std::cout << "State: " << state << std::endl;
            });

        pc->onGatheringStateChange([pc, channel](rtc::PeerConnection::GatheringState state) {
            std::cout << "Gathering State: " << state << std::endl;
            if (state == rtc::PeerConnection::GatheringState::Complete) {
                auto description = pc->localDescription();
                json message = {{"type", description->typeString()},
                    {"sdp", std::string(description.value())}};
                std::cout << "Here is our offer:\n"
                          << message << std::endl;

                std::cout << "Please copy/paste the answer provided by the other side:"
                          << std::endl;
                std::string sdp;
                std::getline(std::cin, sdp);

                json j = json::parse(sdp);
                rtc::Description answer(j["sdp"].get<std::string>(), j["type"].get<std::string>());
                pc->setRemoteDescription(answer);
                std::cout << "The remote description has been set!" << std::endl;

                std::thread([channel]() {
                    for (;; std::this_thread::sleep_for(std::chrono::seconds(1))) {
                        std::cout << "channel->isOpen() = " << std::boolalpha << channel->isOpen()
                                  << std::endl;
                    }
                });
            }
        });

        pc->setLocalDescription();

        // Idle forever
        for (;; std::this_thread::sleep_for(std::chrono::seconds(1))) {
        }
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
}
